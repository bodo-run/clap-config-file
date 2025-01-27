# `ClapConfigFile`

A Rust crate that unifies **command-line arguments** (powered by [Clap](https://github.com/clap-rs/clap)) and **configuration file** loading (YAML, JSON, or TOML).

## What is this?

Consider you have a CLI tool that accepts a series of arguments:

```bash
my-tool --port 8080 --verbose --database-url="sqlite://mem.db"
```

And you want to be able to specify a configuration file to make it easier to run the tool:

```yaml
# my-tool.yaml
port: 8080
verbose: true
database_url: sqlite://mem.db
```

After integrating `ClapConfigFile`, your tool will be able to load the config file automatically:

```bash
my-tool --port 4242 # still possible to provide CLI arguments
# My tool now will get the rest of the config from the "my-tool.yaml" file
```

## Usage Example

```rust
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile)]
#[config_file_name("my-tool.yaml")]
struct AppConfig {
    // Available in both CLI & config, with CLI override by default
    #[config_arg("port", short = "p", default_value = "8080")]
    pub port: u16,

    // CLI-only. A field in config is ignored.
    #[cli_only]
    #[config_arg("verbose")]
    pub verbose: bool,

    // Config-only. CLI flag is ignored.
    #[config_only]
    pub database_url: String,

    // Example of a multi-value field. e.g. --ignored-files foo.txt --ignored-files bar.txt
    #[cli_and_config]
    #[config_arg("ignore")]
    #[multi_value_behavior("extend")] // can be "overwrite"
    pub ignored_files: Vec<String>,
}

fn main() {
    // Provide a default config file name (e.g., "my-tool.yaml").
    // This will be used in file discovery unless `--config-file` or `--no-config` is set.
    let config = AppConfig::parse_with_default_file_name("my-tool.yaml");

    println!("Port: {}", config.port);
    println!("Verbose: {}", config.verbose);
    println!("Database URL: {}", config.database_url);
    println!("Ignored files: {:?}", config.ignored_files);
}
```

Now it's possible to run the tool with the config file:

```bash
my-tool --port 4242
Port: 4242
Verbose: false
Database URL: sqlite://mem.db
Ignored files: []
```

## Attributes

Use these attributes on your struct fields to control sources and behaviors:

- `#[cli_only]`
  - Field can only be set via CLI
- `#[config_only]`
  - Field can only be set by the configuration file (or raw config string)
- `#[cli_and_config]`
  - Field can be set by both CLI and config. The CLI overrides if both are present
- `#[config_arg("arg_name", ...)]`
  - Additional metadata for the CLI side (similar to Clap's `#[clap(...)]`)
  - Set the long option name, short option name, default values, etc.
- `#[multi_value_behavior("extend" | "overwrite")]`
  - For `Vec<T>` fields
  - `extend` merges config and CLI-supplied items
  - `overwrite` replaces config items if CLI has any values

## Automatically Added CLI Flags

These flags are automatically added to the CLI parser:

1. `--config-file <FILE>`
   - Overrides default discovery. Loads from `<FILE>` directly
2. `--no-config`
   - If set, no file is loaded. Only CLI arguments and thier defaults apply
3. `--config <RAW>`
   - If used, parse `<RAW>` as YAML/JSON/TOML and merge with any discovered file or CLI arguments
4. `--help`
   - Show help text

> [!NOTE]
> Seeking feedback if `--config` should be smart enough to figure out if a path to a config file is provided and load it accordingly. See [the open issue](https://github.com/bodo-run/clap-config-file/issues/1)

## Error Handling

- Multiple Config Files: If conflicting files (`my-tool.yaml`, `my-tool.json`) exist in the same directory, the crate exits with an error
- Missing Required Fields: If a required config-only field is not found in the file, or if the user omits a required CLI field, an error is reported
- Invalid Format: If `my-tool.yaml` is invalid YAML syntax, the crate reports a parse error
- No File Found: If no file is found during walk-up and the field is required, the crate errors out (unless `--no-config` is given, in which case it's valid if the user provides enough CLI arguments)

## Configuration File Discovery

By default, if you provide `"my-tool.yaml"` as the file name:

1. The crate starts in the current directory
2. It checks if `my-tool.yaml` exists
3. If not found, it walks up parent directories until it reaches the root
4. If a file is found, it's loaded
5. If multiple files with different extensions (e.g., `my-tool.json`, `my-tool.toml`) are found in the same directory, the crate throws an error unless explicitly overridden by `--config-file`

## Example Workflows

```bash
# Normal run with auto-discovery
$ my_tool
Using config file: /path/to/my-tool.yaml

# Walk-up discovery
$ cd repo/subdir
$ my_tool
Found config file in: /path/to/repo/my-tool.yaml

# Skip config file
$ my_tool --no-config
Not loading config file. CLI-only mode.

# Explicit config file
$ my_tool --config-file /some/other/config.toml

# Raw config
$ my_tool --config '{"port": 8081, "database_url": "sqlite://mem.db"}'
```

## Examples

Check out the [`examples` directory](examples) for a comprehensive examples.

Run the example:

```bash
# Basic run with config auto-discovery
cargo run --example basic

# Override settings via CLI
cargo run --example advanced -- \
    --port 3001 \
    --overwrite-list "cli_item_a" \
    --overwrite-list "cli_item_b"

# Skip config file entirely
cargo run --example advanced -- --no-config --port 9999
```

## Contributing

1. Fork this repository
2. Create a feature branch
3. Submit a pull request describing your changes

Issues and pull requests are welcomed.

## License

[MIT](LICENSE)
