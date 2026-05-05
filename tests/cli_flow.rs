use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn init_commit_log_flow() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    Command::cargo_bin("ics")
        .unwrap()
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::write(root.join("note.md"), "hello").unwrap();

    Command::cargo_bin("ics")
        .unwrap()
        .current_dir(root)
        .args(["commit", "-m", "first note"])
        .assert()
        .success();

    let out = Command::cargo_bin("ics")
        .unwrap()
        .current_dir(root)
        .args(["log"])
        .output()
        .expect("run log");
    assert!(out.status.success(), "log stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("first note"),
        "unexpected log output: {stdout}"
    );
}
