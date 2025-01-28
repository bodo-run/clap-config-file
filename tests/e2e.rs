use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::write;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// Builds the specified example so that `target/debug/examples/<example>` exists.
fn build_example(example: &str) {
    // This runs from the workspace root (the same dir as your Cargo.toml).
    Command::new("cargo")
        .args(["build", "--example", example])
        .assert()
        .success();
}

/// Returns the path to the compiled example binary.
/// By default it lands in `target/debug/examples/<example>`.
fn example_bin(example: &str) -> PathBuf {
    Path::new("target")
        .join("debug")
        .join("examples")
        .join(example)
}

// ------------------------------------------------------------------

#[test]
fn basic_no_args() {
    build_example("basic");
    let tmp = tempdir().unwrap();

    // Copy the compiled binary's path
    let bin = example_bin("basic");
    // We'll run this actual binary from the ephemeral temp dir
    // so that it won't find any other config outside.
    let mut cmd = Command::new(&bin);
    cmd.current_dir(tmp.path());

    cmd.assert().success().stdout(
        predicate::str::contains("Final config:")
            .and(predicate::str::contains("port: 8080"))
            .and(predicate::str::contains("database_url: \"\""))
            .and(predicate::str::contains("debug: None"))
            .and(predicate::str::contains("No config file used")),
    );
}

#[test]
fn basic_with_config_discovered() {
    build_example("basic");
    let tmp = tempdir().unwrap();

    // Write the config file in the temp dir
    write(
        tmp.path().join("app-config.yaml"),
        r#"
port: 9999
database_url: "postgres://my-host/db"
debug: true
"#,
    )
    .unwrap();

    let bin = example_bin("basic");
    let mut cmd = Command::new(&bin);
    cmd.current_dir(tmp.path());

    cmd.assert().success().stdout(
        predicate::str::contains("Final config:")
            .and(predicate::str::contains("port: 9999"))
            .and(predicate::str::contains(
                "database_url: \"postgres://my-host/db\"",
            ))
            .and(predicate::str::contains("debug: Some(true)"))
            .and(predicate::str::contains("Loaded config from:")),
    );
}

#[test]
fn basic_with_config_plus_cli_override() {
    build_example("basic");
    let tmp = tempdir().unwrap();

    write(
        tmp.path().join("app-config.yaml"),
        r#"
port: 7777
database_url: "sqlite://somewhere"
debug: false
"#,
    )
    .unwrap();

    let bin = example_bin("basic");
    let mut cmd = Command::new(&bin);
    cmd.current_dir(tmp.path())
        // pass CLI args after the binary name
        .arg("--port=5555");

    cmd.assert().success().stdout(
        predicate::str::contains("Final config:")
            .and(predicate::str::contains("port: 5555")) // CLI override
            .and(predicate::str::contains(
                "database_url: \"sqlite://somewhere\"",
            ))
            .and(predicate::str::contains("debug: Some(false)")),
    );
}

#[test]
fn basic_no_config_flag() {
    build_example("basic");
    let tmp = tempdir().unwrap();

    // Even if a config is present...
    write(
        tmp.path().join("app-config.yaml"),
        r#"
port: 1234
database_url: "anything"
debug: true
"#,
    )
    .unwrap();

    // ...we explicitly disable config loading with `--no-config`.
    let bin = example_bin("basic");
    let mut cmd = Command::new(&bin);
    cmd.current_dir(tmp.path()).arg("--no-config");

    cmd.assert().success().stdout(
        predicate::str::contains("Final config:")
            .and(predicate::str::contains("port: 8080"))
            .and(predicate::str::contains("database_url: \"\""))
            .and(predicate::str::contains("debug: None"))
            .and(predicate::str::contains("No config file used")),
    );
}

// Repeat the same approach for your Advanced tests:
#[test]
fn advanced_no_args() {
    build_example("advanced");
    let tmp = tempdir().unwrap();

    let bin = example_bin("advanced");
    let mut cmd = Command::new(&bin);
    cmd.current_dir(tmp.path());

    cmd.assert().success().stdout(
        predicate::str::contains("AdvancedConfig")
            .and(predicate::str::contains("server_port: 8080"))
            .and(predicate::str::contains("database_url: \"\""))
            .and(predicate::str::contains("debug: Some(false)"))
            .and(predicate::str::contains("extend_list: []")),
    );
}

// ...and so forth for all your advanced_* tests:
