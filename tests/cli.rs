//! Exercise the installed binary's public, headless contract, not library mocks.
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_taskotto"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn snapshot_json_has_real_system_fields() {
    let output = run(&["--snapshot", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(data["logical_cpus"].as_u64().unwrap() > 0);
    assert!(data["memory"]["total"].as_u64().unwrap() > 0);
    assert!(!data["processes"].as_array().unwrap().is_empty());
}

#[test]
fn cli_rejects_invalid_combinations_without_launching_gui() {
    for args in [
        vec!["--json"],
        vec!["--theme", "not-a-rice"],
        vec!["--refresh-ms", "20"],
        vec!["--install-desktop", "--snapshot"],
    ] {
        let output = run(&args);
        assert!(!output.status.success(), "unexpected success for {args:?}");
        assert!(!output.stderr.is_empty());
    }
    let output = run(&["--version"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        format!("Taskotto {}", env!("CARGO_PKG_VERSION"))
    );
    let help = run(&["--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--snapshot"));
    assert!(String::from_utf8_lossy(&help.stdout).contains("--check-ricepack"));
}

#[test]
fn ricepack_can_be_validated_headlessly_and_invalid_files_are_rejected() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/ricepacks/sea-glass.toml"
    );
    let output = run(&["--check-ricepack", path]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Sea Glass"));
    let invalid = run(&["--check-ricepack", "/does-not-exist/taskotto.toml"]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("invalid RicePack"));
    let conflict = run(&["--ricepack", path, "--snapshot"]);
    assert!(!conflict.status.success());
}

struct TestDataHome(PathBuf);

impl TestDataHome {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "taskotto desktop test {} {nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDataHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn installer_creates_a_reusable_user_local_desktop_entry() {
    let home = TestDataHome::new();
    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_taskotto"))
            .arg("--install-desktop")
            .env("XDG_DATA_HOME", &home.0)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let app_id = "io.github.copypasteuser.taskotto";
    let desktop =
        fs::read_to_string(home.0.join(format!("applications/{app_id}.desktop"))).unwrap();
    assert!(desktop.contains("Name=Taskotto"));
    let executable = env!("CARGO_BIN_EXE_taskotto");
    let exec = if executable.contains(' ') {
        format!("Exec=\"{executable}\"")
    } else {
        format!("Exec={executable}")
    };
    assert!(desktop.contains(&exec));
    assert!(desktop.contains("Icon=io.github.copypasteuser.taskotto"));
    assert!(
        home.0
            .join(format!("icons/hicolor/scalable/apps/{app_id}.svg"))
            .exists()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_taskotto"))
        .arg("--install-desktop")
        .env("XDG_DATA_HOME", "relative/data")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("XDG_DATA_HOME must be absolute"));
}
