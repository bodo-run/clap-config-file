use clap::{arg, Parser};
use clap_config_file::ClapConfigFile;

/// This struct demonstrates usage of the ClapConfigFile derive with various attributes.
///
/// - `#[cli_and_config]`: Field can come from both CLI and config. CLI overrides if both exist.
/// - `#[config_only]`: Field only comes from config file (or raw config data).
/// - `#[cli_only]`: Field only comes from CLI.
/// - `#[multi_value_behavior]`: Controls how `Vec` fields merge between config and CLI.
#[derive(ClapConfigFile, Parser, Debug)]
#[clap(trailing_var_arg = true)]
struct AdvancedConfig {
    /// Example of a field that can come from both CLI and config, with CLI taking precedence.
    ///
    /// We also rename it with `#[config_arg(name = "port")]` and give a short `-p`, plus a default of 8080.
    #[cli_and_config]
    #[config_arg(name = "port", short = 'p', default_value = "8080")]
    pub server_port: u16,

    /// A boolean that can appear in both config and CLI (CLI wins if both).
    /// Demonstrates how booleans become `--debug` on the CLI.
    #[cli_and_config]
    #[config_arg(name = "debug")]
    pub debug: bool,

    /// Field that is only available in the config file. Cannot be set via CLI.
    #[config_only]
    pub database_url: String,

    /// Another config-only field (must appear in config or have a default).
    #[config_only]
    pub special_secret: String,

    /// Nested config-only data (any valid serde type).
    #[config_only]
    pub extra_settings: serde_json::Value,

    /// A list of strings that merges config + CLI lists (`Extend`).
    ///
    /// Usage on CLI: `--extend-list item1 --extend-list item2`
    #[cli_and_config]
    #[config_arg(name = "extend-list")]
    #[multi_value_behavior(Extend)]
    pub extend_list: Vec<String>,

    /// A list of strings that overwrites config if CLI is given (`Overwrite`).
    ///
    /// Usage on CLI: `--overwrite-list item1 --overwrite-list item2`
    #[cli_and_config]
    #[config_arg(name = "overwrite-list")]
    #[multi_value_behavior(Overwrite)]
    pub overwrite_list: Vec<String>,

    /// Any additional commands or arguments
    #[clap(last = true)]
    pub commands: Vec<String>,
}

fn main() {
    // This uses `.parse_with_default_file_name("advanced-config.yaml")` to auto-discover
    // the config file by walking up parent directories, unless overridden by CLI flags.
    let config = AdvancedConfig::parse_with_default_file_name("advanced-config.yaml");

    println!("Final merged config:\n{:#?}", config);

    // Example usage:
    //   $ cargo run --example advanced -- --port 3001 --overwrite-list CLI_item additional_command
    //
    // That overrides the port and the overwrite_list. If advanced-config.yaml
    // has `database_url` etc., that remains unless overridden or suppressed.
    if !config.commands.is_empty() {
        println!("\nReceived commands: {:?}", config.commands);
    }
}
