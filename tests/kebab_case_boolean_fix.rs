use clap_config_file::ClapConfigFile;
use std::fs;
use tempfile::TempDir;
use assert_cmd::prelude::*;
use std::process::Command;

/// Test boolean fields with underscores (converted to kebab-case) work correctly with config files
#[test]
fn test_boolean_kebab_case_config_integration() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    
    let test_code = r#"
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name = "test-config"]
struct TestConfig {
    #[config_arg()]
    pub tree_header: bool,
    
    #[config_arg()]
    pub line_numbers: bool,
    
    #[config_arg()]
    pub simple: bool,
}

fn main() {
    let cfg = TestConfig::parse();
    println!("tree_header: {}", cfg.tree_header);
    println!("line_numbers: {}", cfg.line_numbers);
    println!("simple: {}", cfg.simple);
}
"#;
    
    fs::create_dir(dir.path().join("src"))?;
    fs::write(dir.path().join("src/main.rs"), test_code)?;
    let current_dir = std::env::current_dir()?;
    fs::write(dir.path().join("Cargo.toml"), format!(r#"
[package]
name = "test-kebab-case-bool"
version = "0.1.0"
edition = "2021"

[dependencies]
clap-config-file = {{ path = "{}" }}
clap = {{ version = "4.2", features = ["derive"] }}
config = "0.13"
serde = {{ version = "1.0", features = ["derive"] }}
"#, current_dir.display()))?;
    
    // Test with kebab-case field names in config file
    fs::write(dir.path().join("test-config.yaml"), r#"
tree-header: true
line-numbers: false
simple: true
"#)?;
    
    let output = Command::new("cargo")
        .arg("run")
        .current_dir(dir.path())
        .output()?;
    
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    
    if !output.status.success() {
        panic!("Command failed with stderr: {}\nstdout: {}", stderr, stdout);
    }
    
    // Check that boolean values from config file are correctly loaded
    assert!(stdout.contains("tree_header: true"), "tree_header should be true from config, got: {}", stdout);
    assert!(stdout.contains("line_numbers: false"), "line_numbers should be false from config, got: {}", stdout);
    assert!(stdout.contains("simple: true"), "simple should be true from config, got: {}", stdout);
    
    Ok(())
}