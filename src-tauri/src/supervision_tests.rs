use super::supervision::*;
use serde_json::json;
use std::{collections::VecDeque, sync::Mutex, time::Duration};

fn config() -> Configuration {
    Configuration { node: "C:/node.exe".into(), home: "C:/fixture".into(), service: "moveup/desk".into() }
}
fn response(exit: i32, value: serde_json::Value) -> Response {
    Response { exit, stdout: serde_json::to_vec(&value).unwrap() }
}
fn ack(intent: &str) -> Response {
    response(0, json!({"name":"moveup/desk", "suspended":true, "scope":"login",
        "loginId":"L", "intentId":intent, "acknowledgedAt":"now", "holds":1}))
}
fn status(mode: &str, intent: &str) -> Response {
    response(0, json!({"state":"known", "name":"moveup/desk", "supervision":{
        "mode":mode, "ownedChild":false, "observedLive":false, "activeCandidate":null,
        "promotedCandidate":null, "misses":0, "demoted":[],
        "extra":"compatible", "suspension":{"loginId":"L", "holds":[
            {"intentId":"user", "acknowledgedAt":"before"},
            {"intentId":intent, "acknowledgedAt":"now"}]}}}))
}
struct Fake { replies: Mutex<VecDeque<Response>>, calls: Mutex<Vec<(String, String)>> }
impl Fake {
    fn new(replies: Vec<Response>) -> Self {
        Self { replies: Mutex::new(replies.into()), calls: Mutex::new(vec![]) }
    }
}
#[async_trait::async_trait]
impl Transport for Fake {
    async fn invoke(&self, _: &Configuration, action: &str, intent: &str,
        _: Duration) -> Result<Response, String> {
        self.calls.lock().unwrap().push((action.into(), intent.into()));
        self.replies.lock().unwrap().pop_front().ok_or("unknown".into())
    }
}

#[tokio::test]
async fn acknowledged_quit_needs_no_status() {
    let fake = Fake::new(vec![ack("quit")]);
    assert!(suspend(&config(), "quit", &fake, DEADLINE).await.is_ok());
    assert_eq!(*fake.calls.lock().unwrap(), vec![("suspend".into(), "quit".into())]);
}
#[tokio::test]
async fn lost_acknowledgement_reconciles_own_hold() {
    let fake = Fake::new(vec![response(3, json!({"outcome":"unknown"})), status("suspended", "quit")]);
    assert!(suspend(&config(), "quit", &fake, DEADLINE).await.is_ok());
    assert_eq!(fake.calls.lock().unwrap()[1].0, "supervision");
}
#[tokio::test]
async fn previous_user_suspension_does_not_authorize_this_quit() {
    let fake = Fake::new(vec![response(3, json!({})), status("suspended", "other"), ack("quit")]);
    assert!(suspend(&config(), "quit", &fake, DEADLINE).await.is_ok());
    assert_eq!(fake.calls.lock().unwrap().len(), 3);
    assert!(fake.calls.lock().unwrap().iter().all(|(_, i)| i == "quit"));
}
#[tokio::test]
async fn unknown_inner_mode_never_authorizes_quit() {
    let fake = Fake::new(vec![response(3, json!({})), status("unknown", "quit")]);
    assert!(suspend(&config(), "quit", &fake, DEADLINE).await.is_err());
    assert_eq!(fake.calls.lock().unwrap().len(), 4);
}
#[tokio::test]
async fn transport_errors_are_bounded_unknown() {
    let fake = Fake::new(vec![]);
    assert!(suspend(&config(), "quit", &fake, DEADLINE).await.is_err());
    assert_eq!(fake.calls.lock().unwrap().len(), 4);
}
struct Hanging;
#[async_trait::async_trait]
impl Transport for Hanging {
    async fn invoke(&self, _: &Configuration, _: &str, _: &str, _: Duration) -> Result<Response, String> {
        std::future::pending().await
    }
}
#[tokio::test]
async fn timeout_has_a_total_deadline() {
    let start = std::time::Instant::now();
    assert!(suspend(&config(), "quit", &Hanging, Duration::from_millis(25)).await.is_err());
    assert!(start.elapsed() < Duration::from_secs(1));
}
#[test]
fn malformed_incomplete_wrong_service_or_scope_are_unknown() {
    for value in [json!({}), json!({"name":"other", "suspended":true}),
        json!({"state":"known", "name":"moveup/desk", "supervision":{}}),
        json!({"state":"unknown", "name":"moveup/desk"})] {
        assert!(!confirms(&response(0, value), "supervision", "moveup/desk", "quit"));
    }
    for bytes in [b"{".to_vec(), vec![b' '; MAX_OUTPUT + 1], b"{} {}".to_vec()] {
        assert!(!confirms(&Response {exit:0, stdout:bytes}, "suspend", "moveup/desk", "quit"));
    }
    let mut wrong = ack("quit");
    wrong.exit = 1;
    assert!(!confirms(&wrong, "suspend", "moveup/desk", "quit"));
    assert!(!confirms(&ack("other"), "suspend", "moveup/desk", "quit"));
}
#[test]
fn ordinary_bridge_and_partial_configuration() {
    assert!(Configuration::from_env(|_| None).unwrap().is_none());
    assert!(Configuration::from_env(|k| (k == "DESK_SUPERVISION").then(|| "bridge".into())).unwrap().is_none());
    for (key, value) in [("DESK_SUPERVISION", "pm3"), ("PM3_SERVICE", "moveup/desk"),
        ("DESK_PM3_NODE", "relative"), ("DESK_SUPERVISION", "typo")] {
        assert!(Configuration::from_env(|k| (k == key).then(|| value.into())).is_err());
    }
}
#[test]
fn structured_arguments_preserve_spaces_and_never_use_general_status() {
    let mut cfg = config();
    cfg.home = "C:/fixture with spaces".into();
    let args = cfg.arguments("suspend", "quit");
    assert!(args[0].contains("fixture with spaces"));
    assert_eq!(&args[1..4], &["suspend", "moveup/desk", "--intent"]);
    assert_eq!(cfg.arguments("supervision", "quit")[1], "supervision");
}

#[test]
fn incomplete_status_even_with_matching_hold_is_unknown() {
    let good = status("suspended", "quit");
    assert!(confirms(&good, "supervision", "moveup/desk", "quit"));
    for field in ["mode", "ownedChild", "observedLive", "activeCandidate", "promotedCandidate", "misses", "demoted"] {
        let mut v: serde_json::Value = serde_json::from_slice(&good.stdout).unwrap();
        v["supervision"].as_object_mut().unwrap().remove(field);
        assert!(!confirms(&response(0, v), "supervision", "moveup/desk", "quit"), "{field}");
    }
}

#[test]
fn duplicate_clicks_and_failed_retry_share_one_intent() {
    let gate = std::sync::Arc::new(QuitGate::new());
    let workers: Vec<_> = (0..16).map(|_| {
        let gate = gate.clone();
        std::thread::spawn(move || (gate.begin(), gate.intent().to_string()))
    }).collect();
    let results: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|(allowed, _)| *allowed).count(), 1);
    assert!(results.iter().all(|(_, id)| id == gate.intent()));
    let first = gate.intent().to_string();
    gate.retry();
    assert!(gate.begin());
    assert_eq!(gate.intent(), first);
}
