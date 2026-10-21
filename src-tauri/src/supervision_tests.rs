use super::supervision::*;
use std::{collections::VecDeque, sync::Mutex, time::Duration};

fn config() -> Configuration {
    Configuration { node: "C:/node.exe".into(), home: "C:/fixture".into(), service: "moveup/desk".into() }
}
fn response(exit: i32, text: &str) -> Response { Response { exit, stdout: text.as_bytes().to_vec() } }
struct Fake { replies: Mutex<VecDeque<Response>>, calls: Mutex<Vec<String>> }
impl Fake {
    fn new(replies: Vec<Response>) -> Self { Self { replies: Mutex::new(replies.into()), calls: Mutex::new(vec![]) } }
}
#[async_trait::async_trait]
impl Transport for Fake {
    async fn invoke(&self, _: &Configuration, action: &str, _: &str, _: Duration) -> Result<Response, String> {
        self.calls.lock().unwrap().push(action.into());
        self.replies.lock().unwrap().pop_front().ok_or("transport failed".into())
    }
}
#[tokio::test]
async fn ordinary_stop_is_one_supported_command_without_suspend_or_retry() {
    for text in ["desk: stopped\n", "moveup/desk: stopped"] {
        let fake = Fake::new(vec![response(0, text)]);
        assert!(stop(&config(), &fake, DEADLINE).await.is_ok());
        assert_eq!(*fake.calls.lock().unwrap(), vec!["stop"]);
    }
}
#[tokio::test]
async fn refusal_unknown_or_accidental_clean_exit_never_confirm_stop() {
    for reply in [response(1, "desk: stopped"), response(0, "desk: running"),
        response(0, "desk: exited (code 0)"), response(0, "{}"), response(0, "other: stopped")] {
        let fake = Fake::new(vec![reply]);
        assert!(stop(&config(), &fake, DEADLINE).await.is_err());
        assert_eq!(*fake.calls.lock().unwrap(), vec!["stop"]);
    }
}
#[test]
fn malformed_advanced_or_wrong_service_evidence_is_not_an_ordinary_stop() {
    for text in ["", "{}", r#"{"name":"moveup/desk","suspended":true}"#, "other: stopped",
        "desk: stopped\nother: running"] {
        assert!(!confirms(&response(0, text), "stop", "moveup/desk", ""));
    }
    assert!(!confirms(&Response { exit: 0, stdout: vec![b'x'; MAX_OUTPUT + 1] }, "stop", "moveup/desk", ""));
    assert!(confirms(&response(0, r#"{"name":"moveup/desk","state":"stopped"}"#), "describe", "moveup/desk", ""));
    assert!(!confirms(&response(0, r#"{"name":"other/desk","state":"stopped"}"#), "describe", "moveup/desk", ""));
}
struct Hanging;
#[async_trait::async_trait]
impl Transport for Hanging {
    async fn invoke(&self, _: &Configuration, _: &str, _: &str, _: Duration) -> Result<Response, String> {
        std::future::pending().await
    }
}
#[tokio::test]
async fn stop_timeout_has_one_total_deadline() {
    let start = std::time::Instant::now();
    assert!(stop(&config(), &Hanging, Duration::from_millis(25)).await.is_err());
    assert!(start.elapsed() < Duration::from_secs(1));
}
#[test]
fn bridge_partial_configuration_and_autostart_ownership_remain_distinct() {
    assert!(Configuration::from_env(|_| None).unwrap().is_none());
    assert!(Configuration::from_env(|k| (k == "DESK_SUPERVISION").then(|| "bridge".into())).unwrap().is_none());
    assert!(!pm3_owns_autostart(|_| None));
    assert!(!pm3_owns_autostart(|k| (k == "DESK_SUPERVISION").then(|| "bridge".into())));
    for (key, value) in [("DESK_SUPERVISION", "pm3"), ("PM3_SERVICE", "moveup/desk"),
        ("DESK_PM3_NODE", "relative"), ("DESK_SUPERVISION", "typo")] {
        assert!(Configuration::from_env(|k| (k == key).then(|| value.into())).is_err());
        assert!(pm3_owns_autostart(|k| (k == key).then(|| value.into())));
    }
}
#[test]
fn ordinary_arguments_are_structured_and_only_describe_uses_json() {
    let mut cfg = config(); cfg.home = "C:/fixture with spaces".into();
    assert!(cfg.arguments("stop", "")[0].contains("fixture with spaces"));
    assert_eq!(&cfg.arguments("stop", "")[1..], &["stop", "moveup/desk"]);
    assert_eq!(&cfg.arguments("start", "")[1..], &["start", "moveup/desk"]);
    assert_eq!(&cfg.arguments("describe", "")[1..], &["describe", "moveup/desk", "--json"]);
}

#[test]
fn manual_release_preserves_existing_public_launcher_login_activation() {
    let launcher = std::path::Path::new("C:/Users/user name/.pm3/releases/launch.mjs");
    for prefix in ["\"C:/stable/node.exe\" ", "conhost --headless \"C:/stable/node.exe\" "] {
        assert!(is_pm3_login_activation(&format!("{prefix}\"{}\" start moveup/desk", launcher.display()), launcher));
    }
    for command in ["desk.exe --minimized", "node wrong/launch.mjs start moveup/desk",
        "node \"C:/Users/user name/.pm3/releases/launch.mjs\" start other/desk"] {
        assert!(!is_pm3_login_activation(command, launcher));
    }
}
#[test]
fn quoted_cmd_login_activation_preserves_only_the_selected_final_start() {
    let launcher = std::path::Path::new("C:/fixture/pm3/releases/launch.mjs");
    // Accepted static Run argv, with disposable fixture paths replacing the user paths.
    let candidate = r#"C:\Windows\System32\conhost.exe --headless C:\Windows\System32\cmd.exe /d /s /c "cd /d C:\fixture\config && C:\fixture\node.exe C:\fixture\pm3\releases\launch.mjs start moveup/desk""#;
    assert!(is_pm3_login_activation(candidate, launcher));
    let direct = r#"C:\fixture\node.exe C:\fixture\pm3\releases\launch.mjs start moveup/desk"#;
    assert!(is_pm3_login_activation(direct, launcher));
    for invalid in [
        candidate.replace("fixture\\pm3", "foreign\\pm3"),
        candidate.replace("start moveup/desk", "start other/desk"),
        candidate.replace("start moveup/desk", "start moveup/desk && echo unrelated"),
        format!("{candidate} && echo unrelated"),
        candidate.replace("start moveup/desk", &format!("start moveup/desk && {direct}")),
        candidate.trim_end_matches('"').to_string(),
        format!("{direct} && echo unrelated"),
        format!("{direct} | echo unrelated"),
    ] {
        assert!(!is_pm3_login_activation(&invalid, launcher), "unexpectedly recognized {invalid}");
    }
}
#[test]
fn duplicate_quit_clicks_are_serialized_and_failure_allows_retry() {
    let gate = std::sync::Arc::new(QuitGate::new());
    let workers: Vec<_> = (0..16).map(|_| {
        let gate = gate.clone(); std::thread::spawn(move || gate.begin())
    }).collect();
    let allowed: usize = workers.into_iter().map(|w| usize::from(w.join().unwrap())).sum();
    assert_eq!(allowed, 1);
    gate.retry(); assert!(gate.begin()); assert!(!gate.begin());
}
