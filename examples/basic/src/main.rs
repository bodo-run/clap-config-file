use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
struct AppConfig {
    // Both CLI & config
    #[cli_and_config]
    #[config_arg(name = "port", short = 'p', default_value = "8080")]
    pub port: u16,

    // CLI-only field
    #[cli_only]
    #[config_arg(name = "verbose", short = 'v')]
    pub verbose: bool,

    // Config-only field
    #[config_only]
    pub database_url: String,

    // Multi-value field with extend behavior
    #[cli_and_config]
    #[config_arg(name = "ignored-files")]
    // #[multi_value_behavior("extend")] // TODO: Implement this
    pub ignored_files: Vec<String>,
}

fn main() {
    let config = AppConfig::parse_with_default_file_name("app-config.yaml");

    println!("Configuration loaded successfully:");
    println!("Port: {}", config.port);
    println!("Verbose mode: {}", config.verbose);
    println!("Database URL: {}", config.database_url);
    println!("Ignored files: {:?}", config.ignored_files);
}
