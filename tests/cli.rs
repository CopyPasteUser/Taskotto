use std::process::Command;

fn command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_taskotto"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn help_and_version_work_without_a_display_server() {
    let help = command(&["--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--snapshot"));
    let version = command(&["--version"]);
    assert!(version.status.success());
}

#[test]
fn headless_json_contains_real_processes() {
    let output = command(&["--snapshot", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["memory"]["total"].as_u64().unwrap() > 0);
    assert!(json["logical_cpus"].as_u64().unwrap() > 0);
    assert!(!json["processes"].as_array().unwrap().is_empty());
}

#[test]
fn invalid_cli_options_are_rejected() {
    assert!(!command(&["--json"]).status.success());
    assert!(!command(&["--refresh-ms", "10"]).status.success());
    assert!(!command(&["--theme", "no-such-theme"]).status.success());
}
