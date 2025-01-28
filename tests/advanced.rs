use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn extend_list_merging() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    std::fs::write(dir.path().join("advanced-config.yaml"), 
        "extend_list: [\"config_item\"]\noverwrite_list: [\"config_item\"]\nspecial_secret: \"secret\"\nextra_settings: { nesting_level: 3, allow_guest: false }")?;

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
        "overwrite_list: [\"config_item\"]\nspecial_secret: \"secret\"\nextra_settings: { nesting_level: 3, allow_guest: false }")?;

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
        "special_secret: \"from_config\"\nextra_settings: { nesting_level: 3, allow_guest: false }",
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
