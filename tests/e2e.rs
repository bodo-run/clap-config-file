use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

/// End-to-end test demonstrating how fields merge between CLI and config.
#[test]
fn test_extend_behavior() {
    // Create a temporary YAML config file containing a port, database_url, etc.
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

    // We'll run the 'basic' example with a few CLI overrides to see if merging works as expected.
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

    // Check for the final merged output:
    // port 8080 overrides 3000, and ignored_files should contain "config.log", "foo.txt", plus the new "bar.txt" & "baz.log".
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Port: 8080"))
        .stdout(predicate::str::contains("config.log"))
        .stdout(predicate::str::contains("foo.txt"))
        .stdout(predicate::str::contains("bar.txt"))
        .stdout(predicate::str::contains("baz.log"));
}
