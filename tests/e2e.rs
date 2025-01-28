use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_extend_behavior() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "port: 3000\nignored_files:\n  - config_item\n").unwrap();

    // We'll run the "basic" example, passing a CLI override to see if it extends.
    let mut cmd = Command::cargo_bin("basic").unwrap();
    cmd.arg("--config-file")
        .arg(f.path())
        .arg("--port")
        .arg("9999")
        .arg("--ignored-files")
        .arg("cli_item_a")
        .arg("--ignored-files")
        .arg("cli_item_b");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("port: 9999"))
        .stdout(predicate::str::contains("config_item"))
        .stdout(predicate::str::contains("cli_item_a"))
        .stdout(predicate::str::contains("cli_item_b"));
}
