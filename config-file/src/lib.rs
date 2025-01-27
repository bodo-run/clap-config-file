use clap::Parser;
use serde::{Deserialize, Serialize};

/// A struct for capturing global config-file related CLI flags.
/// This includes:
///   - `--no-config` (skip reading any config file)
///   - `--config-file <path>` (explicit config file path)
///   - `--config <raw>` (raw config string in YAML/JSON/TOML)
///   - leftover arguments
#[derive(Parser, Debug, Default, Deserialize, Serialize)]
#[command(trailing_var_arg = true)]
pub struct ConfigFile {
    /// If `true`, the config file will not be loaded at all.
    #[arg(long = "no-config", default_value_t = false)]
    pub no_config: bool,

    /// Optional path to a configuration file (YAML/JSON/TOML).
    #[arg(long = "config-file")]
    pub config_file: Option<std::path::PathBuf>,

    /// Optional raw configuration string, which can be YAML, JSON, or TOML.
    #[arg(long = "config")]
    pub raw_config: Option<String>,

    /// Any leftover arguments after parsing known flags.
    #[arg(last = true)]
    pub leftover: Vec<String>,
}

impl ConfigFile {
    /// Loads a `ConfigFile` from CLI arguments, optionally with a fallback default config string.
    ///
    /// - If `--no-config` is set, returns the parsed struct immediately.
    /// - If `--config` is set, tries to parse that raw string first.
    /// - If `--config-file` is set, tries to load that file.
    /// - If neither is set, tries to parse the `default_config` string.
    ///
    /// Returns the final `ConfigFile`.
    pub fn load(default_config: &str) -> Self {
        let args = if cfg!(test) {
            if let Ok(args) = std::env::var("ARGS_TEST") {
                Self::parse_from(args.split_whitespace())
            } else {
                Self::parse()
            }
        } else {
            Self::parse()
        };

        if args.no_config {
            return args;
        }

        // Try to load from raw config first
        if let Some(raw_config) = args.raw_config.as_ref() {
            if let Ok(config) = serde_yaml::from_str(raw_config) {
                return config;
            }
        }

        // Then try to load from config file
        if let Some(config_path) = args.config_file.as_ref() {
            match std::fs::read_to_string(config_path) {
                Ok(contents) => {
                    if let Ok(config) = serde_yaml::from_str(&contents) {
                        return config;
                    }
                }
                Err(e) => eprintln!("Failed to read config file: {}", e),
            }
        }

        // Finally try default config
        if let Ok(config) = serde_yaml::from_str(default_config) {
            return config;
        }

        args
    }
}
