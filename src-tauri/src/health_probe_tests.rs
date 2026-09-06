//! Tests for [`crate::health_probe`] (E014-T03).
//!
//! The three cases the plan names — alive-but-never-serving must fail, serving
//! then exiting early must fail, both conditions met must pass — plus the shadow
//! paths around them: a body that is not JSON, an empty body, a request that
//! errors, and a process already gone when the probe starts.
//!
//! Time is faked so a thirty-second window costs nothing, but the seam that
//! actually talks HTTP is exercised against a real socket, not a stub, because a
//! probe that agrees with an imagined server proves nothing.

use std::cell::Cell;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::rc::Rc;
use std::time::Duration;

use crate::health_probe::{
    probe, Clock, DisplayApi, HttpDisplayApi, ProbeConfig, ProcessLiveness, Verdict,
};
use crate::last_known_good::ProbeOutcome;

/// Virtual clock shared by the fake process, API and clock.
#[derive(Clone, Default)]
struct TestTime(Rc<Cell<u64>>);

impl TestTime {
    fn millis(&self) -> u64 {
        self.0.get()
    }
}

struct FakeClock(TestTime);

impl Clock for FakeClock {
    fn now_millis(&self) -> u64 {
        self.0.millis()
    }

    fn sleep(&mut self, duration: Duration) {
        self.0 .0.set(self.0.millis() + duration.as_millis() as u64);
    }
}

/// A process that is alive until `dies_at_millis`, and forever when that is `None`.
struct FakeProcess {
    time: TestTime,
    dies_at_millis: Option<u64>,
}

impl ProcessLiveness for FakeProcess {
    fn is_alive(&mut self) -> bool {
        self.dies_at_millis
            .is_none_or(|dies_at| self.time.millis() < dies_at)
    }
}

/// An endpoint that returns `body` from `serves_from_millis` onward.
struct FakeApi {
    time: TestTime,
    serves_from_millis: Option<u64>,
    body: String,
    calls: usize,
}

impl FakeApi {
    fn serving_json_from(time: &TestTime, from_millis: u64) -> Self {
        Self {
            time: time.clone(),
            serves_from_millis: Some(from_millis),
            body: r#"{"state":"Sitting"}"#.to_string(),
            calls: 0,
        }
    }

    fn never_serving(time: &TestTime) -> Self {
        Self {
            time: time.clone(),
            serves_from_millis: None,
            body: String::new(),
            calls: 0,
        }
    }

    fn serving_body_from(time: &TestTime, from_millis: u64, body: &str) -> Self {
        Self {
            time: time.clone(),
            serves_from_millis: Some(from_millis),
            body: body.to_string(),
            calls: 0,
        }
    }
}

impl DisplayApi for FakeApi {
    fn fetch(&mut self, _timeout: Duration) -> Result<String, String> {
        self.calls += 1;
        match self.serves_from_millis {
            Some(from) if self.time.millis() >= from => Ok(self.body.clone()),
            _ => Err("connection refused".to_string()),
        }
    }
}

/// Runs a probe with the default config over a fake process and API.
fn run(dies_at_millis: Option<u64>, api: FakeApi, time: &TestTime) -> (Verdict, usize) {
    let mut process = FakeProcess {
        time: time.clone(),
        dies_at_millis,
    };
    let mut clock = FakeClock(time.clone());
    let mut api = api;
    let verdict = probe(&mut process, &mut api, &mut clock, &ProbeConfig::default());
    (verdict, api.calls)
}

#[test]
fn a_process_that_stays_alive_but_never_serves_is_not_good() {
    let time = TestTime::default();
    let (verdict, _) = run(None, FakeApi::never_serving(&time), &time);

    assert_eq!(verdict, Verdict::NeverServedJson);
    assert_eq!(verdict.outcome(), ProbeOutcome::Failed);
    assert_eq!(time.millis(), 30_000, "the probe watched the whole window");
}

#[test]
fn a_process_that_serves_then_exits_before_thirty_seconds_is_not_good() {
    let time = TestTime::default();
    let (verdict, _) = run(
        Some(12_000),
        FakeApi::serving_json_from(&time, 0),
        &time,
    );

    assert_eq!(
        verdict,
        Verdict::ExitedEarly {
            alive_for: Duration::from_secs(12),
            served: true,
        }
    );
    assert_eq!(verdict.outcome(), ProbeOutcome::Failed);
}

#[test]
fn alive_for_the_window_and_serving_json_is_good() {
    let time = TestTime::default();
    let (verdict, _) = run(None, FakeApi::serving_json_from(&time, 3_000), &time);

    assert_eq!(verdict, Verdict::Good);
    assert_eq!(verdict.outcome(), ProbeOutcome::Good);
}

#[test]
fn a_process_already_gone_when_the_probe_starts_is_not_good() {
    let time = TestTime::default();
    let (verdict, calls) = run(Some(0), FakeApi::serving_json_from(&time, 0), &time);

    assert_eq!(
        verdict,
        Verdict::ExitedEarly {
            alive_for: Duration::ZERO,
            served: false,
        }
    );
    assert_eq!(calls, 0, "a dead process is not asked to serve");
}

#[test]
fn a_two_hundred_that_is_not_json_does_not_count_as_serving() {
    let time = TestTime::default();
    let (verdict, _) = run(
        None,
        FakeApi::serving_body_from(&time, 0, "<!doctype html><html>error</html>"),
        &time,
    );

    assert_eq!(verdict, Verdict::NeverServedJson);
}

#[test]
fn an_empty_body_does_not_count_as_serving() {
    let time = TestTime::default();
    let (verdict, _) = run(None, FakeApi::serving_body_from(&time, 0, ""), &time);

    assert_eq!(verdict, Verdict::NeverServedJson);
}

#[test]
fn serving_only_after_the_window_would_have_closed_is_not_good() {
    let time = TestTime::default();
    let (verdict, _) = run(None, FakeApi::serving_json_from(&time, 60_000), &time);

    assert_eq!(verdict, Verdict::NeverServedJson);
}

#[test]
fn the_endpoint_is_not_polled_again_once_it_has_served() {
    let time = TestTime::default();
    let (verdict, calls) = run(None, FakeApi::serving_json_from(&time, 0), &time);

    assert_eq!(verdict, Verdict::Good);
    assert_eq!(calls, 1, "one success settles the JSON half of D5");
}

#[test]
fn a_shorter_window_is_honoured() {
    let time = TestTime::default();
    let mut process = FakeProcess {
        time: time.clone(),
        dies_at_millis: Some(5_000),
    };
    let mut clock = FakeClock(time.clone());
    let mut api = FakeApi::serving_json_from(&time, 0);
    let config = ProbeConfig {
        alive_requirement: Duration::from_secs(3),
        ..ProbeConfig::default()
    };

    let verdict = probe(&mut process, &mut api, &mut clock, &config);

    assert_eq!(verdict, Verdict::Good, "it outlived a three-second window");
}

/// Serves one canned HTTP response on loopback and returns the address.
fn serve_once(response: &'static str) -> SocketAddr {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).expect("bind");
    let addr = listener.local_addr().expect("local addr");
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    addr
}

#[test]
fn the_http_client_returns_the_body_of_a_two_hundred() {
    let addr = serve_once(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"state\":\"Sitting\"}",
    );
    let mut api = HttpDisplayApi::new(addr);

    let body = api.fetch(Duration::from_secs(5)).expect("fetch");

    assert_eq!(body, r#"{"state":"Sitting"}"#);
}

#[test]
fn the_http_client_reports_a_non_two_hundred_as_an_error() {
    let addr = serve_once("HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\nboom");
    let mut api = HttpDisplayApi::new(addr);

    let err = api.fetch(Duration::from_secs(5)).expect_err("must not pass");

    assert!(err.contains("500"), "unexpected error: {err}");
}

#[test]
fn the_http_client_reports_a_closed_port_as_an_error() {
    // Bind, read the address, drop the listener: nothing answers there now.
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).expect("bind");
    let addr = listener.local_addr().expect("local addr");
    drop(listener);
    let mut api = HttpDisplayApi::new(addr);

    assert!(api.fetch(Duration::from_millis(500)).is_err());
}
