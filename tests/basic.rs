use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn with_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    std::fs::write(
        dir.path().join("app-config.yaml"),
        "port: 8080\ndatabase_url: \"postgres://localhost:5432/mydb\"",
    )?;

    Command::cargo_bin("basic")?
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("port: 8080"))
        .stdout(predicate::str::contains(
            "database_url: \"postgres://localhost:5432/mydb\"",
        ));

    Ok(())
}

#[test]
fn cli_override_port() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    std::fs::write(
        dir.path().join("app-config.yaml"),
        "port: 8080\ndatabase_url: \"postgres://localhost:5432/mydb\"",
    )?;

    Command::cargo_bin("basic")?
        .current_dir(dir.path())
        .arg("--port")
        .arg("9090")
        .assert()
        .success()
        .stdout(predicate::str::contains("port: 9090"))
        .stdout(predicate::str::contains(
            "database_url: \"postgres://localhost:5432/mydb\"",
        ));

    Ok(())
}

#[test]
fn no_config_uses_defaults() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("basic")?
        .arg("--no-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("port: 8080"))
        .stdout(predicate::str::contains("database_url: \"\"")); // Default String is empty

    Ok(())
}
