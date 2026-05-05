use assert_cmd::Command;

#[test]
fn help_exits_zero() {
    Command::cargo_bin("ics")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}
