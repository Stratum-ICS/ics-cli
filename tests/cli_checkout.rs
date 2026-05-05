use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn checkout_restores_from_head() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    Command::cargo_bin("ics")
        .unwrap()
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::write(root.join("note.md"), "v1").unwrap();
    Command::cargo_bin("ics")
        .unwrap()
        .current_dir(root)
        .args(["commit", "-m", "snap"])
        .assert()
        .success();

    fs::write(root.join("note.md"), "edited").unwrap();
    Command::cargo_bin("ics")
        .unwrap()
        .current_dir(root)
        .args(["checkout", "note.md"])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(root.join("note.md")).unwrap(), "v1");
}

#[test]
fn checkout_requires_paths_or_all() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    Command::cargo_bin("ics")
        .unwrap()
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::write(root.join("note.md"), "x").unwrap();
    Command::cargo_bin("ics")
        .unwrap()
        .current_dir(root)
        .args(["commit", "-m", "x"])
        .assert()
        .success();

    Command::cargo_bin("ics")
        .unwrap()
        .current_dir(root)
        .args(["checkout"])
        .assert()
        .failure();
}
