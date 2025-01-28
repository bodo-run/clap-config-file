use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::path::Path;
use std::process::Command;

#[test]
fn with_config_file() -> Result<(), Box<dyn std::error::Error>> {
    // path to basic example
    let dir = Path::new("examples/basic");

    Command::cargo_bin("basic")?
        .current_dir(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Final config:"))
        .stdout(predicate::str::contains("port: 8080"))
        .stdout(predicate::str::contains("postgres://localhost:5432/mydb"))
        .stdout(predicate::str::contains("Loaded config from:"));

    Ok(())
}

#[test]
fn cli_override_port() -> Result<(), Box<dyn std::error::Error>> {
    // path to basic example
    let dir = Path::new("examples/basic");

    Command::cargo_bin("basic")?
        .current_dir(dir)
        .arg("--port")
        .arg("9090")
        .assert()
        .success()
        .stdout(predicate::str::contains("Final config:"))
        .stdout(predicate::str::contains("port: 9090"))
        .stdout(predicate::str::contains("postgres://localhost:5432/mydb"))
        .stdout(predicate::str::contains("Loaded config from:"));

    Ok(())
}

#[test]
fn no_config_uses_defaults() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("basic")?
        .arg("--no-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("Final config:"))
        .stdout(predicate::str::contains("port: 8080"))
        .stdout(predicate::str::contains("database_url: None"))
        .stdout(predicate::str::contains("No config file used"));

    Ok(())
}
