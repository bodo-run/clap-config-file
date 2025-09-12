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
        .stdout(predicate::str::contains("port: 9090")) // Changed from 8080 to 9090
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

#[test]
fn config_overrides_default_github_issue_8() -> Result<(), Box<dyn std::error::Error>> {
    // This test specifically validates the fix for GitHub issue #8
    // Config file values should override default_value attributes
    
    let dir = Path::new("examples/basic");
    
    // The config file has port: 9090
    // The struct has #[config_arg(default_value = "8080")]
    // Expected: config value (9090) should override default (8080)
    
    Command::cargo_bin("basic")?
        .current_dir(dir)
        .arg("--config-file")
        .arg("app-config.yaml")
        .assert()
        .success()
        .stdout(predicate::str::contains("port: 9090"))
        .stdout(predicate::str::contains("port: 8080").not());

    Ok(())
}
