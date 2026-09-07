//! relay_client_tests.rs — the desk relay client against a real socket.
//!
//! Every test runs an in-process tokio-tungstenite server on port 0, so the
//! handshake, the framing and the close codes are the genuine article rather
//! than a mock of what we assume they do. Only the reconnect delay is faked
//! ([`FakeSleeper`]), and the three timeouts are shortened through
//! [`RelayConfig`] — nothing else is stubbed.

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};

use crate::relay_client::{
    should_run, spawn, ws_url, ClientDeps, ClientHandle, RelayConfig, Sleeper, SnapshotSource,
};
use crate::relay_status::{RelayState, RelayStatus};
use crate::remote_protocol::{
    Envelope, MessageType, CLOSE_REVOKED, CLOSE_SHEDDING, PROTOCOL_VERSION,
};

type ServerWs = WebSocketStream<TcpStream>;

// ─── Fakes ──────────────────────────────────────────────────────────────────

/// Records what the client would have waited, and blocks forever once `cap`
/// delays have been observed so a failing test cannot spin the CPU.
struct FakeSleeper {
    slept: Arc<Mutex<Vec<Duration>>>,
    cap: usize,
}

impl Sleeper for FakeSleeper {
    fn sleep(&self, d: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let mut v = self.slept.lock().unwrap_or_else(|e| e.into_inner());
        v.push(d);
        if v.len() >= self.cap {
            Box::pin(std::future::pending())
        } else {
            Box::pin(async {})
        }
    }
}

struct FixedSnapshot;

impl SnapshotSource for FixedSnapshot {
    fn snapshot(&self) -> Value {
        json!({ "event": "snapshot", "payload": { "session": { "state": "Sitting" } } })
    }
}

// ─── Harness ────────────────────────────────────────────────────────────────

async fn serve<F, Fut>(handler: F) -> (SocketAddr, Arc<AtomicUsize>)
where
    F: Fn(usize, ServerWs) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let conns = Arc::new(AtomicUsize::new(0));
    let counter = conns.clone();
    let handler = Arc::new(handler);
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let n = counter.fetch_add(1, Ordering::SeqCst);
            let h = handler.clone();
            tokio::spawn(async move {
                if let Ok(ws) = accept_async(stream).await {
                    h(n, ws).await;
                }
            });
        }
    });
    (addr, conns)
}

struct Fixture {
    handle: ClientHandle,
    status: Arc<Mutex<RelayStatus>>,
    ws_tx: broadcast::Sender<String>,
    slept: Arc<Mutex<Vec<Duration>>>,
    conns: Arc<AtomicUsize>,
}

impl Fixture {
    fn state(&self) -> RelayState {
        self.status.lock().unwrap_or_else(|e| e.into_inner()).state
    }

    fn sleeps(&self) -> Vec<Duration> {
        self.slept
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn attempts(&self) -> usize {
        self.conns.load(Ordering::SeqCst)
    }
}

fn start(addr: SocketAddr, conns: Arc<AtomicUsize>, tune: impl FnOnce(&mut RelayConfig)) -> Fixture {
    let mut cfg = RelayConfig::new(format!("http://{addr}"), "desk-1", "mu_d_test", "0.6.0");
    cfg.welcome_timeout = Duration::from_millis(500);
    tune(&mut cfg);
    let ws_tx = broadcast::channel::<String>(8).0;
    let status = Arc::new(Mutex::new(RelayStatus::default()));
    let slept = Arc::new(Mutex::new(Vec::new()));
    let handle = spawn(
        cfg,
        ClientDeps {
            ws_tx: ws_tx.clone(),
            status: status.clone(),
            sleeper: Arc::new(FakeSleeper {
                slept: slept.clone(),
                cap: 4,
            }),
            snapshot: Arc::new(FixedSnapshot),
        },
    );
    Fixture {
        handle,
        status,
        ws_tx,
        slept,
        conns,
    }
}

/// Reads the next envelope, failing the test rather than hanging.
async fn recv(ws: &mut ServerWs) -> Envelope<Value> {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
            .await
            .expect("server read timed out")
            .expect("stream ended")
            .expect("ws error");
        if let Message::Text(t) = msg {
            return serde_json::from_str(&t).expect("envelope");
        }
    }
}

async fn send(ws: &mut ServerWs, msg_type: MessageType, payload: Value) {
    let env = Envelope {
        v: PROTOCOL_VERSION,
        msg_type,
        id: "srv-1".to_string(),
        ts: 0,
        payload,
    };
    let json = serde_json::to_string(&env).expect("serialize");
    ws.send(Message::Text(json.into())).await.expect("send");
}

fn welcome_payload(viewer_count: u32) -> Value {
    json!({
        "role": "desk",
        "desk_id": "desk-1",
        "desk_online": true,
        "viewer_count": viewer_count,
        "snapshot": null,
        "snapshot_ts": null,
    })
}

/// Polls `f` until it holds or ~2 s pass.
async fn until(mut f: impl FnMut() -> bool) -> bool {
    for _ in 0..200 {
        if f() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    f()
}

// ─── happy path ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn hello_welcome_snapshot_then_forwards_broadcasts() {
    let (seen_tx, mut seen_rx) = tokio::sync::mpsc::unbounded_channel();
    let (addr, conns) = serve(move |_n, mut ws| {
        let seen = seen_tx.clone();
        async move {
            let hello = recv(&mut ws).await;
            let _ = seen.send(hello);
            send(&mut ws, MessageType::Welcome, welcome_payload(2)).await;
            while let Some(Ok(Message::Text(t))) = ws.next().await {
                let _ = seen.send(serde_json::from_str(&t).expect("envelope"));
            }
        }
    })
    .await;

    let fx = start(addr, conns, |_| {});

    let hello = seen_rx.recv().await.expect("hello");
    assert_eq!(hello.msg_type, MessageType::Hello);
    assert_eq!(hello.v, PROTOCOL_VERSION);
    assert_eq!(hello.payload["role"], "desk");
    assert_eq!(hello.payload["desk_id"], "desk-1");
    assert_eq!(hello.payload["token"], "mu_d_test");
    assert_eq!(hello.payload["client"]["app"], "moveup-desk");

    let snapshot = seen_rx.recv().await.expect("snapshot");
    assert_eq!(snapshot.msg_type, MessageType::Event);
    assert_eq!(snapshot.payload["event"], "snapshot");

    assert!(until(|| fx.state() == RelayState::Online).await);
    assert_eq!(
        fx.status
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .viewers_online,
        2
    );

    let _ = fx
        .ws_tx
        .send(r#"{"event":"desk:state-changed","payload":{"state":"Standing"}}"#.to_string());
    let forwarded = seen_rx.recv().await.expect("forwarded");
    assert_eq!(forwarded.msg_type, MessageType::Event);
    assert_eq!(forwarded.payload["event"], "desk:state-changed");
    assert_eq!(forwarded.payload["payload"]["state"], "Standing");

    fx.handle.shutdown().await;
}

// ─── nil: nothing to connect with ───────────────────────────────────────────

#[test]
fn should_run_needs_both_the_flag_and_a_token() {
    assert!(should_run(true, Some("mu_d_x")));
    assert!(!should_run(true, None));
    assert!(!should_run(true, Some("")));
    assert!(!should_run(false, Some("mu_d_x")));
    assert_eq!(RelayStatus::default().state, RelayState::Disabled);
}

// ─── error: terminal close codes ────────────────────────────────────────────

#[tokio::test]
async fn revoked_close_stops_the_loop_without_reconnecting() {
    let (addr, conns) = serve(|_n, mut ws| async move {
        let _ = recv(&mut ws).await;
        let _ = ws
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Library(CLOSE_REVOKED),
                reason: "revoked".into(),
            })))
            .await;
        let _ = ws.close(None).await;
    })
    .await;

    let fx = start(addr, conns, |_| {});
    assert!(until(|| fx.state() == RelayState::Revoked).await);

    // The proof is the absence of a retry: no delay was ever requested and the
    // server saw exactly one connection after a generous wait.
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(fx.state(), RelayState::Revoked);
    assert!(fx.sleeps().is_empty(), "a terminal close must not back off");
    assert_eq!(fx.attempts(), 1, "a terminal close must not reconnect");
    assert!(fx.handle.is_finished(), "the task must end");
}

#[tokio::test]
async fn retryable_close_backs_off_and_doubles() {
    let (addr, conns) = serve(|_n, mut ws| async move {
        let _ = recv(&mut ws).await;
        let _ = ws
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Library(CLOSE_SHEDDING),
                reason: "shedding".into(),
            })))
            .await;
        let _ = ws.close(None).await;
    })
    .await;

    let fx = start(addr, conns, |_| {});
    assert!(until(|| fx.sleeps().len() >= 3).await, "expected three retries");

    let sleeps = fx.sleeps();
    for (i, base) in [1.0_f64, 2.0, 4.0].iter().enumerate() {
        let got = sleeps[i].as_secs_f64();
        assert!(
            got >= base * 0.8 - 1e-9 && got <= base * 1.2 + 1e-9,
            "retry {i} waited {got}s, expected {base}s ±20 %"
        );
    }
    assert!(fx.attempts() >= 3);
    fx.handle.shutdown().await;
}

#[tokio::test]
async fn drop_mid_hello_counts_as_one_failed_attempt() {
    let (addr, conns) = serve(|_n, mut ws| async move {
        // Read the hello, then vanish without ever sending a welcome.
        let _ = recv(&mut ws).await;
        drop(ws);
    })
    .await;

    let fx = start(addr, conns, |_| {});
    assert!(until(|| fx.sleeps().len() >= 2).await, "expected a second attempt");

    let sleeps = fx.sleeps();
    assert!(sleeps[0].as_secs_f64() <= 1.2 + 1e-9);
    assert!(sleeps[1].as_secs_f64() >= 1.6, "backoff must double: {:?}", sleeps);
    assert!(fx.attempts() >= 2);
    fx.handle.shutdown().await;
}

// ─── empty: an idle connection ──────────────────────────────────────────────

#[tokio::test]
async fn idle_connection_sends_exactly_one_ping_per_interval() {
    let (seen_tx, mut seen_rx) = tokio::sync::mpsc::unbounded_channel();
    let (addr, conns) = serve(move |_n, mut ws| {
        let seen = seen_tx.clone();
        async move {
            let _ = recv(&mut ws).await;
            send(&mut ws, MessageType::Welcome, welcome_payload(0)).await;
            while let Some(Ok(Message::Text(t))) = ws.next().await {
                let env: Envelope<Value> = serde_json::from_str(&t).expect("envelope");
                let _ = seen.send(env.msg_type);
            }
        }
    })
    .await;

    let fx = start(addr, conns, |c| {
        c.ping_every = Duration::from_millis(120);
        c.pong_timeout = Duration::from_secs(30);
    });

    assert_eq!(seen_rx.recv().await.expect("snapshot"), MessageType::Event);
    assert_eq!(seen_rx.recv().await.expect("ping"), MessageType::Ping);
    // No traffic at all until the next interval elapses.
    assert!(
        tokio::time::timeout(Duration::from_millis(60), seen_rx.recv())
            .await
            .is_err(),
        "a second ping arrived inside one interval"
    );
    fx.handle.shutdown().await;
}

#[tokio::test]
async fn missing_pong_drops_and_reconnects() {
    let (addr, conns) = serve(|_n, mut ws| async move {
        let _ = recv(&mut ws).await;
        send(&mut ws, MessageType::Welcome, welcome_payload(0)).await;
        // Swallow everything, answer nothing.
        while ws.next().await.is_some() {}
    })
    .await;

    let fx = start(addr, conns, |c| {
        c.ping_every = Duration::from_millis(40);
        c.pong_timeout = Duration::from_millis(1);
    });

    assert!(until(|| fx.attempts() >= 2).await, "a dead peer must be dropped");
    assert!(!fx.sleeps().is_empty());
    fx.handle.shutdown().await;
}

// ─── control surface ────────────────────────────────────────────────────────

#[tokio::test]
async fn stop_ends_the_task_and_reports_disabled() {
    let (addr, conns) = serve(|_n, mut ws| async move {
        let _ = recv(&mut ws).await;
        send(&mut ws, MessageType::Welcome, welcome_payload(1)).await;
        while ws.next().await.is_some() {}
    })
    .await;

    let fx = start(addr, conns, |_| {});
    assert!(until(|| fx.state() == RelayState::Online).await);

    fx.handle.stop();
    assert!(until(|| fx.state() == RelayState::Disabled).await);
    assert_eq!(fx.attempts(), 1);
    fx.handle.shutdown().await;
}

#[tokio::test]
async fn restart_opens_a_second_connection_without_backing_off() {
    let (addr, conns) = serve(|_n, mut ws| async move {
        let _ = recv(&mut ws).await;
        send(&mut ws, MessageType::Welcome, welcome_payload(0)).await;
        while ws.next().await.is_some() {}
    })
    .await;

    let fx = start(addr, conns, |_| {});
    assert!(until(|| fx.state() == RelayState::Online).await);

    fx.handle.restart();
    assert!(until(|| fx.attempts() >= 2).await, "restart must reconnect");
    assert!(
        fx.sleeps().is_empty(),
        "restart must not wait out a backoff"
    );
    fx.handle.shutdown().await;
}

// ─── URL shapes ─────────────────────────────────────────────────────────────

#[test]
fn ws_url_upgrades_browser_schemes_and_keeps_socket_ones() {
    assert_eq!(
        ws_url("https://relay.desk.zentala.io", "d1"),
        "wss://relay.desk.zentala.io/v1/desks/d1/ws"
    );
    assert_eq!(
        ws_url("http://127.0.0.1:8787/", "d1"),
        "ws://127.0.0.1:8787/v1/desks/d1/ws"
    );
    assert_eq!(
        ws_url("wss://relay.example/", "d2"),
        "wss://relay.example/v1/desks/d2/ws"
    );
}
