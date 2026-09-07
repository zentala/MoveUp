//! relay_client.rs — the desk's outbound WebSocket to the relay (E022-T05).
//!
//! One tokio task owns one connection: `hello` → `welcome` → a fresh snapshot
//! → every message from the `ws_tx` broadcast forwarded verbatim inside an
//! `event` envelope. The same channel already feeds the LAN server, so the
//! relay is one more `subscribe()` and never a second event producer.
//!
//! Everything the tests need to control is injected: [`Sleeper`] for the
//! reconnect delay, [`SnapshotSource`] for the payload, and the three timeouts
//! on [`RelayConfig`]. Credentials, config flags and IPC are T06's job — this
//! module is handed a token and never reads the credential store itself.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::relay_commands::CommandExecutor;
use crate::relay_status::{should_reconnect, state_for_close_code, Backoff, RelayState, RelayStatus};
use crate::remote_protocol::{
    check_version, ClientInfo, Command, CommandResult, Envelope, ErrorBody, Hello, MessageType,
    Role, Welcome, PROTOCOL_VERSION,
};

/// Sent while the connection is idle so a dead peer is noticed.
pub const PING_EVERY: Duration = Duration::from_secs(25);
/// No `pong` for this long counts as a dead connection.
pub const PONG_TIMEOUT: Duration = Duration::from_secs(60);
/// A `welcome` later than this is a failed attempt.
pub const WELCOME_TIMEOUT: Duration = Duration::from_secs(5);
/// Close code used when the peer vanished without a close frame.
const CLOSE_ABNORMAL: u16 = 1006;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsWrite = SplitSink<WsStream, Message>;
type WsRead = SplitStream<WsStream>;

/// Everything the client needs to reach one room.
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// Relay base URL — `https://`, `http://`, `wss://` or `ws://`.
    pub url: String,
    pub desk_id: String,
    /// Opaque bearer token. Never logged, never put in the URL.
    pub token: String,
    pub app_version: String,
    pub ping_every: Duration,
    pub pong_timeout: Duration,
    pub welcome_timeout: Duration,
}

impl RelayConfig {
    pub fn new(
        url: impl Into<String>,
        desk_id: impl Into<String>,
        token: impl Into<String>,
        app_version: impl Into<String>,
    ) -> Self {
        Self {
            url: url.into(),
            desk_id: desk_id.into(),
            token: token.into(),
            app_version: app_version.into(),
            ping_every: PING_EVERY,
            pong_timeout: PONG_TIMEOUT,
            welcome_timeout: WELCOME_TIMEOUT,
        }
    }
}

/// The socket URL for a desk. `https`/`http` are upgraded to `wss`/`ws` so a
/// user who pastes a browser URL into Settings still gets a working relay.
pub fn ws_url(base: &str, desk_id: &str) -> String {
    let base = base.trim_end_matches('/');
    let base = match base.split_once("://") {
        Some(("https", rest)) => format!("wss://{rest}"),
        Some(("http", rest)) => format!("ws://{rest}"),
        _ => base.to_string(),
    };
    format!("{base}/v1/desks/{desk_id}/ws")
}

/// Whether the client may start at all.
///
/// Both halves matter and neither implies the other: the feature can be on
/// with no credential yet (fresh install), and a credential can outlive the
/// user turning the feature off. Without both the status stays
/// [`RelayState::Disabled`] and no socket is ever opened — a desk that is not
/// registered must not look like one that is merely reconnecting.
pub fn should_run(enabled: bool, token: Option<&str>) -> bool {
    enabled && token.is_some_and(|t| !t.is_empty())
}

/// The reconnect delay, injected so tests do not wait real seconds.
pub trait Sleeper: Send + Sync {
    fn sleep(&self, d: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

/// Production sleeper.
pub struct TokioSleeper;

impl Sleeper for TokioSleeper {
    fn sleep(&self, d: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(tokio::time::sleep(d))
    }
}

/// Produces the `snapshot` [`crate::ws_broadcaster::DisplayEvent`] as JSON.
///
/// A trait rather than the concrete handles so this task does not depend on
/// `SessionManager` — T06 supplies an implementation backed by
/// `remote_display_state::build`.
pub trait SnapshotSource: Send + Sync {
    fn snapshot(&self) -> Value;
}

/// The handles one client task reads and writes.
pub struct ClientDeps {
    pub ws_tx: broadcast::Sender<String>,
    pub status: Arc<Mutex<RelayStatus>>,
    pub sleeper: Arc<dyn Sleeper>,
    pub snapshot: Arc<dyn SnapshotSource>,
}

enum Ctl {
    Stop,
    Restart,
}

/// Why one connection attempt ended.
enum End {
    Stopped,
    Restart,
    Closed(u16),
    Failed(String),
}

/// Control surface for the spawned task.
pub struct ClientHandle {
    ctl: mpsc::UnboundedSender<Ctl>,
    task: tokio::task::JoinHandle<()>,
}

impl ClientHandle {
    /// Ends the task; the status goes to [`RelayState::Disabled`].
    pub fn stop(&self) {
        let _ = self.ctl.send(Ctl::Stop);
    }

    /// Drops the current connection and reconnects immediately, backoff reset.
    /// Called when Settings saves a new relay URL or re-enables the feature.
    pub fn restart(&self) {
        let _ = self.ctl.send(Ctl::Restart);
    }

    pub fn is_finished(&self) -> bool {
        self.task.is_finished()
    }

    /// Stops and waits — used at shutdown and in tests.
    pub async fn shutdown(self) {
        self.stop();
        let _ = self.task.await;
    }
}

/// Starts a client task that ignores every inbound `command`.
///
/// Refusing rather than silently dropping: a viewer that asks a build with no
/// executor still gets a `command_result` saying so, instead of a button that
/// looks like it worked.
pub fn spawn(cfg: RelayConfig, deps: ClientDeps) -> ClientHandle {
    spawn_with_commands(cfg, deps, None)
}

/// Starts the client task with a command executor wired in (E022-T07).
/// Returns immediately.
pub fn spawn_with_commands(
    cfg: RelayConfig,
    deps: ClientDeps,
    commands: Option<Arc<dyn CommandExecutor>>,
) -> ClientHandle {
    let (ctl, ctl_rx) = mpsc::unbounded_channel();
    let task = tokio::spawn(run(cfg, deps, commands, ctl_rx));
    ClientHandle { ctl, task }
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn enter(status: &Arc<Mutex<RelayStatus>>, state: RelayState, error: Option<String>) {
    let mut s = status.lock().unwrap_or_else(|e| e.into_inner());
    s.enter(state, now_ms(), error);
}

/// Connect / fail / back off, until a terminal close code or a stop.
async fn run(
    cfg: RelayConfig,
    deps: ClientDeps,
    commands: Option<Arc<dyn CommandExecutor>>,
    mut ctl_rx: mpsc::UnboundedReceiver<Ctl>,
) {
    let mut backoff = Backoff::new();
    loop {
        enter(&deps.status, RelayState::Connecting, None);
        match session(&cfg, &deps, commands.as_ref(), &mut ctl_rx, &mut backoff).await {
            End::Stopped => {
                enter(&deps.status, RelayState::Disabled, None);
                return;
            }
            End::Restart => {
                backoff.reset();
                continue;
            }
            End::Closed(code) => {
                let state = state_for_close_code(code);
                enter(&deps.status, state, Some(format!("relay closed ({code})")));
                if !should_reconnect(state) {
                    log::warn!("Relay: terminal close {code} — not reconnecting");
                    return;
                }
            }
            End::Failed(err) => {
                log::debug!("Relay: attempt failed: {err}");
                enter(&deps.status, RelayState::Error, Some(err));
            }
        }
        let delay = backoff.next_delay();
        tokio::select! {
            _ = deps.sleeper.sleep(delay) => {}
            ctl = ctl_rx.recv() => match ctl {
                Some(Ctl::Restart) => backoff.reset(),
                _ => {
                    enter(&deps.status, RelayState::Disabled, None);
                    return;
                }
            },
        }
    }
}

/// One connection, from TCP handshake to close.
async fn session(
    cfg: &RelayConfig,
    deps: &ClientDeps,
    commands: Option<&Arc<dyn CommandExecutor>>,
    ctl_rx: &mut mpsc::UnboundedReceiver<Ctl>,
    backoff: &mut Backoff,
) -> End {
    let url = ws_url(&cfg.url, &cfg.desk_id);
    let stream = match tokio_tungstenite::connect_async(&url).await {
        Ok((s, _)) => s,
        Err(e) => return End::Failed(format!("connect: {e}")),
    };
    let (mut write, mut read) = stream.split();

    let hello = Hello {
        role: Role::Desk,
        desk_id: cfg.desk_id.clone(),
        token: cfg.token.clone(),
        client: ClientInfo {
            app: "moveup-desk".to_string(),
            version: cfg.app_version.clone(),
        },
    };
    if let Err(e) = send(&mut write, MessageType::Hello, &hello).await {
        return End::Failed(e);
    }

    let welcome = match tokio::time::timeout(cfg.welcome_timeout, next_envelope(&mut read)).await {
        Err(_) => return End::Failed("welcome timed out".into()),
        Ok(Err(end)) => return end,
        Ok(Ok(env)) => match check_welcome(env) {
            Ok(w) => w,
            Err(e) => return End::Failed(e),
        },
    };

    backoff.reset();
    {
        let mut s = deps.status.lock().unwrap_or_else(|e| e.into_inner());
        s.enter(RelayState::Online, now_ms(), None);
        s.viewers_online = welcome.viewer_count;
    }
    if let Err(e) = send(&mut write, MessageType::Event, &deps.snapshot.snapshot()).await {
        return End::Failed(e);
    }

    pump(cfg, deps, commands, ctl_rx, &mut write, &mut read).await
}

/// Forwards broadcasts out, reads relay messages in, keeps the ping alive.
async fn pump(
    cfg: &RelayConfig,
    deps: &ClientDeps,
    commands: Option<&Arc<dyn CommandExecutor>>,
    ctl_rx: &mut mpsc::UnboundedReceiver<Ctl>,
    write: &mut WsWrite,
    read: &mut WsRead,
) -> End {
    let mut rx = deps.ws_tx.subscribe();
    let mut ping = tokio::time::interval(cfg.ping_every);
    ping.tick().await; // the first tick is immediate; skip it
    let mut last_pong = Instant::now();

    loop {
        tokio::select! {
            ctl = ctl_rx.recv() => return match ctl {
                Some(Ctl::Restart) => End::Restart,
                _ => End::Stopped,
            },
            got = rx.recv() => match got {
                Ok(json) => {
                    let payload: Value = serde_json::from_str(&json).unwrap_or(Value::Null);
                    if let Err(e) = send(write, MessageType::Event, &payload).await {
                        return End::Failed(e);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    log::debug!("Relay: broadcast lagged by {n}, re-sending snapshot");
                    if let Err(e) = send(write, MessageType::Event, &deps.snapshot.snapshot()).await {
                        return End::Failed(e);
                    }
                }
                Err(broadcast::error::RecvError::Closed) => return End::Stopped,
            },
            _ = ping.tick() => {
                if last_pong.elapsed() > cfg.pong_timeout {
                    return End::Failed("no pong within timeout".into());
                }
                if let Err(e) = send(write, MessageType::Ping, &Value::Null).await {
                    return End::Failed(e);
                }
            },
            msg = read.next() => match msg {
                Some(Ok(Message::Text(t))) => match serde_json::from_str::<Envelope<Value>>(&t) {
                    Ok(env) => match env.msg_type {
                        MessageType::Pong => last_pong = Instant::now(),
                        MessageType::Ping => {
                            last_pong = Instant::now();
                            if let Err(e) = send(write, MessageType::Pong, &Value::Null).await {
                                return End::Failed(e);
                            }
                        }
                        MessageType::Command => {
                            if let Err(e) = answer_command(write, &env, commands).await {
                                return End::Failed(e);
                            }
                        }
                        other => log::debug!("Relay: ignoring {}", other.as_str()),
                    },
                    Err(e) => log::debug!("Relay: unparseable frame: {e}"),
                },
                Some(Ok(Message::Close(frame))) => {
                    return End::Closed(frame.map_or(CLOSE_ABNORMAL, |f| u16::from(f.code)))
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => return End::Failed(format!("ws: {e}")),
                None => return End::Closed(CLOSE_ABNORMAL),
            },
        }
    }
}

/// Runs one inbound `command` and writes its `command_result` back.
///
/// Every path answers — an unparseable payload and a build with no executor
/// both produce a failed result rather than silence, because a viewer waiting
/// for an answer it will never get looks exactly like a hung desk.
/// A refused command is never a reason to drop the connection; only a failed
/// *write* is.
async fn answer_command(
    write: &mut WsWrite,
    env: &Envelope<Value>,
    commands: Option<&Arc<dyn CommandExecutor>>,
) -> Result<(), String> {
    let result = match (
        serde_json::from_value::<Command>(env.payload.clone()),
        commands,
    ) {
        (Ok(cmd), Some(exec)) => exec.execute(&Envelope {
            v: env.v,
            msg_type: MessageType::Command,
            id: env.id.clone(),
            ts: env.ts,
            payload: cmd,
        }),
        (Ok(_), None) => failed_result(
            &env.id,
            "commands_unavailable",
            "this desk does not execute remote commands",
        ),
        (Err(e), _) => failed_result(&env.id, "bad_args", format!("unreadable command: {e}")),
    };
    send(write, MessageType::CommandResult, &result).await
}

fn failed_result(command_id: &str, code: &str, message: impl Into<String>) -> CommandResult {
    CommandResult {
        command_id: command_id.to_string(),
        ok: false,
        error: Some(ErrorBody::new(code, message)),
    }
}

/// Reads until a text frame arrives; a close or error ends the attempt.
async fn next_envelope(read: &mut WsRead) -> Result<Envelope<Value>, End> {
    loop {
        match read.next().await {
            Some(Ok(Message::Text(t))) => {
                return serde_json::from_str(&t)
                    .map_err(|e| End::Failed(format!("bad envelope: {e}")))
            }
            Some(Ok(Message::Close(frame))) => {
                return Err(End::Closed(frame.map_or(CLOSE_ABNORMAL, |f| u16::from(f.code))))
            }
            Some(Ok(_)) => continue,
            Some(Err(e)) => return Err(End::Failed(format!("ws: {e}"))),
            None => return Err(End::Closed(CLOSE_ABNORMAL)),
        }
    }
}

/// The first message must be a `welcome` this build can speak.
fn check_welcome(env: Envelope<Value>) -> Result<Welcome, String> {
    check_version(env.v).map_err(|e| e.message)?;
    if env.msg_type != MessageType::Welcome {
        return Err(format!("expected welcome, got {}", env.msg_type.as_str()));
    }
    serde_json::from_value(env.payload).map_err(|e| format!("bad welcome: {e}"))
}

/// Wraps `payload` in the shared envelope and writes it.
async fn send<T: Serialize>(
    write: &mut WsWrite,
    msg_type: MessageType,
    payload: &T,
) -> Result<(), String> {
    let env = Envelope {
        v: PROTOCOL_VERSION,
        msg_type,
        id: uuid::Uuid::new_v4().to_string(),
        ts: now_ms(),
        payload,
    };
    let json = serde_json::to_string(&env).map_err(|e| e.to_string())?;
    write
        .send(Message::Text(json.into()))
        .await
        .map_err(|e| e.to_string())
}
