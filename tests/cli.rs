use assert_cmd::prelude::*; // Add methods on commands
use std::process::Command; // Run programs

#[test]
fn test() {
    let mut cmd = Command::cargo_bin("main").expect("main binary not found for test");
    cmd.arg("p1").arg("test");
    cmd.assert().success();
}
