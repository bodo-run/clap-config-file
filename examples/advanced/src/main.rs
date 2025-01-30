use clap_config_file::ClapConfigFile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ExtraSettings {
    pub nesting_level: i64,
    pub allow_guest: Option<bool>,
}

/// A single struct for both CLI and config file usage
#[derive(ClapConfigFile)]
#[config_file_name = "advanced-config"]
#[config_file_formats = "yaml,toml,json"]
struct AdvancedConfig {
    // Use all the defaults:
    // - config_file_name = "advanced-config"
    // - accept_from = "cli_and_config"
    // - no default value
    #[config_arg()]
    pub database_url: String,

    // DocString handling demonstration: the next line will be printed in help
    /// Port to run the server on
    #[config_arg(
        // Use a different name in the config file
        name = "port",
        // provide a default value
        default_value = "8080",
        // accept from both CLI and config file. This is the default behavior.
        accept_from = "cli_and_config"
    )]
    pub server_port: u16,

    // just specifying --debug will set it to true
    #[config_arg()]
    pub debug: bool,

    // some config files expand environment variables
    // let's say you want the secret never to be set from the CLI so
    // it's not in the history of the CLI
    #[config_arg(accept_from = "config_only")]
    pub special_secret: String,

    // this is a nested struct and can only be set from the config file
    #[config_arg(accept_from = "config_only")]
    pub extra_settings: ExtraSettings,

    // user will extend the list from the config by adding --extend-list=foo1 --extend-list=foo2
    #[config_arg(multi_value_behavior = "extend")]
    pub extend_list: Vec<String>,

    // user will overwrite the list from the config by adding --overwrite-list=foo1 --overwrite-list=foo2
    #[config_arg(multi_value_behavior = "overwrite")]
    pub overwrite_list: Vec<String>,

    // Positional arguments from the CLI, e.g. "file1.txt file2.txt"
    // Positional arguments are always accepted from the CLI only
    // We are assuming those are coming as last arguments. (other cases is not supported)
    #[config_arg(positional)]
    pub paths: Vec<String>,

    // for internal use only. Not configurable from CLI or config file
    // Since config_arg is not used, it will not be included in result of AdvancedConfig::parse_info()
    pub internal_config: String,
}

// a default initializer to handle the config s
impl Default for AdvancedConfig {
    fn default() -> Self {
        let (cfg, _, _) = AdvancedConfig::parse_info();
        Self {
            database_url: cfg.database_url,
            server_port: cfg.server_port,
            debug: cfg.debug,
            special_secret: cfg.special_secret,
            extra_settings: cfg.extra_settings,
            extend_list: cfg.extend_list,
            overwrite_list: cfg.overwrite_list,
            paths: cfg.paths,
            internal_config: "Computed in default initializer".to_string(),
        }
    }
}

fn main() {
    let cfg = AdvancedConfig::default();
    println!("{:#?}", cfg);
}
