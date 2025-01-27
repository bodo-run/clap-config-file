use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_extend_behavior() {
    // Create a temporary YAML config file
    let mut config_file = NamedTempFile::new().unwrap();
    writeln!(
        config_file,
        r#"
port: 3000
database_url: "postgres://localhost:5432/mydb"
ignored_files:
  - "config.log"
  - "foo.txt"
"#
    )
    .unwrap();

    // Use cargo run --example instead of cargo_bin
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--example")
        .arg("basic")
        .arg("--")
        .arg("--config-file")
        .arg(config_file.path())
        .arg("--port")
        .arg("8080")
        .arg("--ignored-files")
        .arg("bar.txt")
        .arg("--ignored-files")
        .arg("baz.log");

    // Run and check output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Port: 8080"))
        .stdout(predicate::str::contains("config.log"))
        .stdout(predicate::str::contains("foo.txt"))
        .stdout(predicate::str::contains("bar.txt"))
        .stdout(predicate::str::contains("baz.log"));
}
