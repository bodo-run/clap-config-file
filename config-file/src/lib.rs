use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Default, Deserialize, Serialize)]
#[command(trailing_var_arg = true)]
pub struct ConfigFile {
    /// If true, the config file will not be loaded
    #[arg(long = "no-config", default_value_t = false)]
    pub no_config: bool,

    /// Path to the configuration file
    #[arg(long = "config-file")]
    pub config_file: Option<std::path::PathBuf>,

    /// Raw configuration string
    #[arg(long = "config")]
    pub raw_config: Option<String>,

    /// Leftover arguments
    #[arg(last = true)]
    pub leftover: Vec<String>,
}

impl ConfigFile {
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
