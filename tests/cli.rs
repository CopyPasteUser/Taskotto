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
    assert!(String::from_utf8_lossy(&help.stdout).contains("--export-ricepack"));
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

#[test]
fn export_ricepack_preserves_user_config_and_refuses_to_overwrite() {
    let home = TestDataHome::new();
    let settings_dir = home.0.join("taskotto");
    fs::create_dir_all(&settings_dir).unwrap();
    let config = settings_dir.join("config.toml");
    let original = "theme = 'windows7'\naccent_hex = '#123ABC'\nlayout_mode = 'sidebar'\nui_spacing = 11.0\nrefresh_ms = 2000\n";
    fs::write(&config, original).unwrap();
    let output_path = home.0.join("saved-rice.toml");
    let export = || {
        Command::new(env!("CARGO_BIN_EXE_taskotto"))
            .arg("--export-ricepack")
            .arg(&output_path)
            .env("XDG_CONFIG_HOME", &home.0)
            .output()
            .unwrap()
    };
    let first = export();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let contents = fs::read_to_string(&output_path).unwrap();
    let pack = taskotto::ricepack::RicePack::load(&output_path).unwrap();
    assert_eq!(pack.base_theme, taskotto::config::Theme::Windows7);
    assert_eq!(pack.appearance.accent.as_deref(), Some("#123ABC"));
    assert_eq!(pack.appearance.spacing, Some(11.0));
    assert_eq!(fs::read_to_string(&config).unwrap(), original);

    let check = run(&["--check-ricepack", output_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let second = export();
    assert!(!second.status.success());
    assert!(String::from_utf8_lossy(&second.stderr).contains("File exists"));
    assert_eq!(fs::read_to_string(&output_path).unwrap(), contents);

    let mixed = run(&[
        "--export-ricepack",
        output_path.to_str().unwrap(),
        "--snapshot",
    ]);
    assert!(!mixed.status.success());
    assert!(String::from_utf8_lossy(&mixed.stderr).contains("must be used on its own"));
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
