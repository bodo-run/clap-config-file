use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn extend_list_merging() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    std::fs::write(dir.path().join("advanced-config.yaml"),
        "extend-list: [\"config_item\"]\noverwrite-list: [\"config_item\"]\nspecial-secret: \"secret\"\nextra-settings:\n  nesting-level: 3\n  allow-guest: false")?;

    Command::cargo_bin("advanced")?
        .current_dir(dir.path())
        .arg("--extend-list")
        .arg("cli_item")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("extend_list: [")
                .and(predicate::str::contains("\"config_item\""))
                .and(predicate::str::contains("\"cli_item\"")),
        );

    Ok(())
}

#[test]
fn overwrite_list_cli() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    std::fs::write(dir.path().join("advanced-config.yaml"),
        "overwrite-list: [\"config_item\"]\nspecial-secret: \"secret\"\nextra-settings:\n  nesting-level: 3\n  allow-guest: false")?;

    Command::cargo_bin("advanced")?
        .current_dir(dir.path())
        .arg("--overwrite-list")
        .arg("cli_item")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("overwrite_list: [")
                .and(predicate::str::contains("\"cli_item\"")),
        );

    Ok(())
}

#[test]
fn config_only_field_from_file() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    std::fs::write(
        dir.path().join("advanced-config.yaml"),
        "special-secret: \"from_config\"",
    )?;

    Command::cargo_bin("advanced")?
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("special_secret: \"from_config\""));

    Ok(())
}

#[test]
fn cli_config_only_field_error() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("advanced")?
        .arg("--special-secret")
        .arg("from_cli")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected argument '--special-secret'",
        ));

    Ok(())
}

#[test]
fn test_positional_args() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    std::fs::write(
        dir.path().join("advanced-config.yaml"),
        "database-url: \"sqlite://test.db\"\n",
    )?;

    Command::cargo_bin("advanced")?
        .current_dir(dir.path())
        .arg("file1.txt")
        .arg("file2.txt")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("paths: [")
                .and(predicate::str::contains("\"file1.txt\""))
                .and(predicate::str::contains("\"file2.txt\"")),
        );

    Ok(())
}

#[test]
fn test_positional_args_cannot_override_cli_only() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    std::fs::write(
        dir.path().join("advanced-config.yaml"),
        "paths: [\"from_config.txt\"]\n",
    )?;

    Command::cargo_bin("advanced")?
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("paths: []"));

    Ok(())
}
