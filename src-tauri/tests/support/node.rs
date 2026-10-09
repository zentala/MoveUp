//! Test-only discovery. Production supervision always requires explicit Node.
use std::path::PathBuf;

pub fn fixture_node() -> PathBuf {
    for key in ["MOVEUP_TEST_NODE", "PM3_DAEMON_NODE", "DESK_PM3_NODE"] {
        if let Some(value) = std::env::var_os(key).filter(|value| !value.is_empty()) {
            let node = PathBuf::from(value);
            assert!(node.is_absolute() && node.is_file(), "{key} must name an absolute Node executable");
            return node.canonicalize().expect("Resolve fixture Node executable");
        }
    }
    let executable = if cfg!(windows) { "node.exe" } else { "node" };
    for directory in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let node = directory.join(executable);
        if node.is_file() { return node.canonicalize().expect("Resolve Node executable from PATH"); }
    }
    panic!("Synthetic transport tests require Node on PATH or absolute MOVEUP_TEST_NODE");
}
