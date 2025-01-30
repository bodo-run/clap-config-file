use clap_config_file::ClapConfigFile;

/// A single struct for both CLI and config file usage
#[derive(ClapConfigFile)]
#[config_file_name = "app-config"]
#[config_file_formats = "yaml,toml,json"]
struct AppConfig {
    /// Database URL
    #[config_arg()]
    pub database_url: Option<String>,

    /// Port to run the server on
    #[config_arg(default_value = "8080")]
    pub port: u16,

    /// Debug mode
    #[config_arg(default_value = "false")]
    pub debug: Option<bool>,
}

fn main() {
    let (cfg, maybe_path, maybe_fmt) = AppConfig::parse_info();
    println!("Final config:\n{:#?}", cfg);

    match maybe_path {
        Some(path) => println!("Loaded config from: {}", path.display()),
        None => println!("No config file used (maybe none found or --no-config)"),
    }
    println!("Detected format: {:?}", maybe_fmt);
}
