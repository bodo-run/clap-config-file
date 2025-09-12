use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Test that config values override default_value attributes for string fields
#[test]
fn config_overrides_string_default() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    // Create a test binary with String field having default
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg(default_value = "default_string")]
    pub my_string: String,
}

fn main() {
    let cfg = TestConfig::parse();
    println!("my_string: {}", cfg.my_string);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "test-default-override"
version = "0.1.0"
edition = "2021"

[dependencies]
clap_config_file = { path = "../.." }
"#)?;
    
    // Test with config file
    fs::write(dir.path().join("test-config.yaml"), "my_string: config_value")?;
    
    Command::new("cargo")
        .arg("run")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("my_string: config_value"));
    
    Ok(())
}

/// Test that CLI overrides both config and default for u16 fields
#[test]
fn cli_overrides_config_and_default_u16() -> Result<(), Box<dyn std::error::Error>> {
    let dir = Path::new("examples/basic");
    
    // Ensure config has port 9090 and default is 8080
    Command::cargo_bin("basic")?
        .current_dir(dir)
        .arg("--port")
        .arg("3000")
        .assert()
        .success()
        .stdout(predicate::str::contains("port: 3000"));
    
    Ok(())
}

/// Test bool field with default_value = "true"
#[test]
fn bool_default_true_overridden_by_config() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg(default_value = "true")]
    pub enabled: bool,
}

fn main() {
    let cfg = TestConfig::parse();
    println!("enabled: {}", cfg.enabled);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "test-bool-default"
version = "0.1.0"
edition = "2021"

[dependencies]
clap_config_file = { path = "../.." }
"#)?;
    
    // Config sets to false, should override default true
    fs::write(dir.path().join("test-config.yaml"), "enabled: false")?;
    
    Command::new("cargo")
        .arg("run")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("enabled: false"));
    
    Ok(())
}

/// Test Option<bool> field with default_value
#[test]
fn option_bool_with_default() -> Result<(), Box<dyn std::error::Error>> {
    let dir = Path::new("examples/basic");
    
    // The debug field is Option<bool> with default_value = "false"
    // Config doesn't set it, so should use default
    Command::cargo_bin("basic")?
        .current_dir(dir)
        .arg("--no-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("debug: Some(\n        false,\n    )").or(predicate::str::contains("debug: Some(false)")));
    
    Ok(())
}

/// Test vector field with comma-separated default values
#[test]
fn vector_field_with_default_values() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg(default_value = "item1,item2,item3")]
    pub items: Vec<String>,
}

fn main() {
    let cfg = TestConfig::parse();
    println!("items: {:?}", cfg.items);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "test-vec-default"
version = "0.1.0"
edition = "2021"

[dependencies]
clap_config_file = { path = "../.." }
"#)?;
    
    // No config file, should use default
    Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--no-config")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"["item1", "item2", "item3"]"#));
    
    Ok(())
}

/// Test vector field where config overrides default
#[test]
fn vector_config_overrides_default() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg(default_value = "default1,default2", multi_value_behavior = "overwrite")]
    pub items: Vec<String>,
}

fn main() {
    let cfg = TestConfig::parse();
    println!("items: {:?}", cfg.items);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "test-vec-override"
version = "0.1.0"
edition = "2021"

[dependencies]
clap_config_file = { path = "../.." }
"#)?;
    
    // Config overrides default
    fs::write(dir.path().join("test-config.yaml"), r#"
items:
  - config1
  - config2
  - config3
"#)?;
    
    Command::new("cargo")
        .arg("run")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"["config1", "config2", "config3"]"#));
    
    Ok(())
}

/// Test extend behavior with vector defaults
#[test]
fn vector_extend_with_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg(default_value = "base1,base2", multi_value_behavior = "extend")]
    pub items: Vec<String>,
}

fn main() {
    let cfg = TestConfig::parse();
    println!("items: {:?}", cfg.items);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "test-vec-extend"
version = "0.1.0"
edition = "2021"

[dependencies]
clap_config_file = { path = "../.." }
"#)?;
    
    // Config adds to default (in extend mode)
    fs::write(dir.path().join("test-config.yaml"), r#"
items:
  - added1
  - added2
"#)?;
    
    Command::new("cargo")
        .arg("run")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"["added1", "added2"]"#));
    
    Ok(())
}

/// Test Option<String> with default value
#[test]
fn option_string_with_default() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg(default_value = "default_value")]
    pub optional: Option<String>,
}

fn main() {
    let cfg = TestConfig::parse();
    println!("optional: {:?}", cfg.optional);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "test-option-default"
version = "0.1.0"
edition = "2021"

[dependencies]
clap_config_file = { path = "../.." }
"#)?;
    
    // No config, should use default
    Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--no-config")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"Some("default_value")"#));
    
    // Config overrides default
    fs::write(dir.path().join("test-config.yaml"), "optional: config_value")?;
    
    Command::new("cargo")
        .arg("run")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"Some("config_value")"#));
    
    Ok(())
}

/// Test empty string default value
#[test]
fn empty_string_default() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg(default_value = "")]
    pub empty: String,
}

fn main() {
    let cfg = TestConfig::parse();
    println!("empty: '{}'", cfg.empty);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "test-empty-default"
version = "0.1.0"
edition = "2021"

[dependencies]
clap_config_file = { path = "../.." }
"#)?;
    
    // No config, should use empty default
    Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--no-config")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("empty: ''"));
    
    Ok(())
}

/// Test numeric field with default parsing
#[test]
fn numeric_default_parsing() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg(default_value = "42")]
    pub number: i32,
    
    #[config_arg(default_value = "3.14")]
    pub float: f64,
}

fn main() {
    let cfg = TestConfig::parse();
    println!("number: {}, float: {}", cfg.number, cfg.float);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "test-numeric-default"
version = "0.1.0"
edition = "2021"

[dependencies]
clap_config_file = { path = "../.." }
"#)?;
    
    // No config, should use defaults
    Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--no-config")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("number: 42, float: 3.14"));
    
    // Config overrides one field
    fs::write(dir.path().join("test-config.yaml"), "number: 100")?;
    
    Command::new("cargo")
        .arg("run")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("number: 100, float: 3.14"));
    
    Ok(())
}

/// Test complete precedence chain: CLI > config > default > type default
#[test]
fn complete_precedence_chain() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg(default_value = "default1")]
    pub field1: String,
    
    #[config_arg(default_value = "default2")]
    pub field2: String,
    
    #[config_arg(default_value = "default3")]
    pub field3: String,
    
    #[config_arg()]
    pub field4: String,  // No default, uses type default (empty string)
}

fn main() {
    let cfg = TestConfig::parse();
    println!("field1: {}", cfg.field1);
    println!("field2: {}", cfg.field2);
    println!("field3: {}", cfg.field3);
    println!("field4: '{}'", cfg.field4);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "test-precedence"
version = "0.1.0"
edition = "2021"

[dependencies]
clap_config_file = { path = "../.." }
"#)?;
    
    // Config overrides field2 and field3
    fs::write(dir.path().join("test-config.yaml"), r#"
field2: config2
field3: config3
"#)?;
    
    // CLI overrides field3
    Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--field3")
        .arg("cli3")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("field1: default1"))  // Uses default
        .stdout(predicate::str::contains("field2: config2"))   // Uses config
        .stdout(predicate::str::contains("field3: cli3"))      // Uses CLI
        .stdout(predicate::str::contains("field4: ''"));       // Uses type default
    
    Ok(())
}

/// Test mixed cli_only, config_only, and cli_and_config fields
#[test]
fn mixed_availability_with_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg(accept_from = "cli_only", default_value = "cli_default")]
    pub cli_field: String,
    
    #[config_arg(accept_from = "config_only", default_value = "config_default")]
    pub config_field: String,
    
    #[config_arg(accept_from = "cli_and_config", default_value = "both_default")]
    pub both_field: String,
}

fn main() {
    let cfg = TestConfig::parse();
    println!("cli_field: {}", cfg.cli_field);
    println!("config_field: {}", cfg.config_field);
    println!("both_field: {}", cfg.both_field);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "test-mixed-availability"
version = "0.1.0"
edition = "2021"

[dependencies]
clap_config_file = { path = "../.." }
"#)?;
    
    // Config tries to set all fields (but cli_field should ignore it)
    fs::write(dir.path().join("test-config.yaml"), r#"
cli_field: ignored_config
config_field: from_config
both_field: from_config
"#)?;
    
    Command::new("cargo")
        .arg("run")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("cli_field: cli_default"))     // Ignores config, uses default
        .stdout(predicate::str::contains("config_field: from_config"))  // Uses config
        .stdout(predicate::str::contains("both_field: from_config"));   // Uses config
    
    // CLI overrides cli_field and both_field
    Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--cli-field")
        .arg("from_cli")
        .arg("--both-field")
        .arg("from_cli")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("cli_field: from_cli"))        // Uses CLI
        .stdout(predicate::str::contains("config_field: from_config"))  // Still uses config
        .stdout(predicate::str::contains("both_field: from_cli"));      // CLI overrides config
    
    Ok(())
} 