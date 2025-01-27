use clap_config_file::ClapConfigFile;

/// This struct covers almost every possible combination of fields.
#[derive(ClapConfigFile, Debug)]
struct AdvancedConfig {
    // server_port can come from both CLI and config; CLI takes precedence if supplied.
    #[cli_and_config]
    #[config_arg(name = "port", short = 'p', default_value = "8080")]
    pub server_port: u16,

    // This is a boolean that can appear in both config and CLI (CLI wins if both).
    // We'll demonstrate how booleans become "--debug" on CLI.
    #[cli_and_config]
    #[config_arg(name = "debug")]
    pub debug: bool,

    // Config-only field (cannot be set via CLI).
    // Must appear in the config file or have a default in the code if you want to avoid errors.
    #[config_only]
    pub database_url: String,

    // Another config-only field (any type that implements serde).
    #[config_only]
    pub special_secret: String,

    // Nested config-only data (serde struct, map, etc.).
    // For advanced nested usage, define a sub-struct with `#[derive(Serialize, Deserialize)]`.
    #[config_only]
    pub extra_settings: serde_json::Value,

    // A list of strings that merges config list + CLI list (multi_value_behavior = Extend).
    // Usage on CLI: `--extend-list item1 --extend-list item2 ...`
    #[cli_and_config]
    #[config_arg(name = "extend-list")]
    #[multi_value_behavior(Extend)]
    pub extend_list: Vec<String>,

    // A list of strings where CLI overwrites config entirely (multi_value_behavior = Overwrite).
    // Usage on CLI: `--overwrite-list item1 --overwrite-list item2 ...`
    #[cli_and_config]
    #[config_arg(name = "overwrite-list")]
    #[multi_value_behavior(Overwrite)]
    pub overwrite_list: Vec<String>,
}

fn main() {
    // This tells the crate to auto-discover "advanced-config.yaml" by walking up parent directories,
    // unless --no-config or --config-file overrides are given.
    let config = AdvancedConfig::parse_with_default_file_name("advanced-config.yaml");

    println!("Final merged config:\n{:#?}", config);

    // Here's a quick demonstration:
    //   $ cargo run --example advanced -- --port 3001 --overwrite-list CLI_item
    //
    // That would override the port to 3001 and the `overwrite_list` to ["CLI_item"].
    // If advanced-config.yaml has "database_url: 'postgres://...'", that remains in place
    // unless we used --no-config or a different file.
}
