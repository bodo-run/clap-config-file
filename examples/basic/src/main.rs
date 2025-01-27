use clap_config_file::ClapConfigFile;

/// A simple example of how to use the ClapConfigFile derive macro.
///
/// - `#[cli_and_config]` means the field can be set from both CLI and config.
/// - `#[cli_only]` means CLI only.
/// - `#[config_only]` means config only.
#[derive(ClapConfigFile)]
struct AppConfig {
    /// Example of a field that can come from both CLI and config.
    /// We rename it with `#[config_arg(name = "port", short = 'p')]` and set a default of 8080.
    #[cli_and_config]
    #[config_arg(name = "port", short = 'p', default_value = "8080")]
    pub port: u16,

    /// This field is CLI-only (no config file).
    #[cli_only]
    #[config_arg(name = "verbose", short = 'v')]
    pub verbose: bool,

    /// This field is config-only (cannot be overridden by CLI).
    #[config_only]
    pub database_url: String,

    /// A multi-value field that can come from both CLI and config.
    /// Demonstrates how lists might be handled. (The default is an extend-like merge.)
    #[cli_and_config]
    #[config_arg(name = "ignored-files")]
    pub ignored_files: Vec<String>,
}

fn main() {
    // Parse the config with a default name `app-config.yaml` that will be discovered if present.
    let config = AppConfig::parse_with_default_file_name("app-config.yaml");

    println!("Configuration loaded successfully:");
    println!("Port: {}", config.port);
    println!("Verbose mode: {}", config.verbose);
    println!("Database URL: {}", config.database_url);
    println!("Ignored files: {:?}", config.ignored_files);
}
