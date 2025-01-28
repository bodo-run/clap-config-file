# `ClapConfigFile`

[`clap`](https://github.com/clap-rs/clap) 🤝 [`config`](https://github.com/rust-cli/config-rs)

Use a single struct for both CLI and config file.

## What is this?

Consider you have a CLI tool that accepts a series of arguments:

```bash
my-tool --port 8080 --debug --database-url="sqlite://mem.db"
```

And you want to be able to specify a configuration file to make it easier to run the tool:

```yaml
# my-tool.yaml
port: 8080
debug: true
database_url: sqlite://mem.db
```

After integrating `ClapConfigFile`, your tool will be able to load the config file automatically:

```bash
my-tool --port 4242 # still possible to provide CLI arguments
# my-tool now will get the rest of the config from the "my-tool.yaml" file
```

## Usage Example

```rust
use clap_config_file::ClapConfigFile;

#[derive(ClapConfigFile, Debug)]
#[config_file_name = "my-tool"]
#[config_file_formats = "yaml,toml,json"]
struct AppConfig {
    // Available in both CLI & config, with CLI override by default
    #[config_arg(default_value = "8080")]
    pub port: u16,

    // CLI-only. A field in config is ignored.
    #[config_arg(accept_from = "cli_only")]
    pub cli_secret: String,

    // Config-only. CLI flag is ignored.
    #[config_arg(accept_from = "config_only")]
    pub database_url: String,

    // Example of a multi-value field, merged from config and CLI.
    // e.g. --ignored-files foo.txt --ignored-files bar.txt
    #[config_arg(multi_value_behavior = "extend")]
    pub ignored_files: Vec<String>,

    // Example of a multi-value field, CLI overwrites config.
    #[config_arg(multi_value_behavior = "overwrite")]
    pub overwrite_list: Vec<String>
}

fn main() {
    // Provide a default config file name (e.g., "my-tool").
    // This will be used in file discovery unless `--config-file` or `--no-config` is set.
    let (config, used_file, format) = AppConfig::parse_info();

    println!("Config: {:?}", config);
}
```

Now it's possible to run the tool with the config file:

```bash
my-tool --port 4242
Config: AppConfig { port: 4242, cli_secret: "", database_url: "sqlite://mem.db", ignored_files: [], overwrite_list: [] }
```

## Attributes

Use these attributes on your struct fields to control sources and behaviors:

- `#[config_arg(accept_from = "cli_only")]`
  - Field can only be set via CLI
- `#[config_arg(accept_from = "config_only")]`
  - Field can only be set by the configuration file
- `#[config_arg(accept_from = "cli_and_config")]` (default)
  - Field can be set by both CLI and config. The CLI overrides if both are present
- `#[config_arg("arg_name", ...)]`
  - Additional metadata for the CLI side (similar to Clap's `#[clap(...)]`)
  - Set the long option name, short option name, default values, etc.
- `#[config_arg(multi_value_behavior = "extend" | "overwrite")]`
  - For `Vec<T>` fields
  - `extend` merges config and CLI-supplied items
  - `overwrite` replaces config items if CLI has any values

**Struct Attributes**

- `#[config_file_name = "my-tool"]`
  - Sets the base name of the config file to search for during auto-discovery. Defaults to "config".
- `#[config_file_formats = "yaml,toml,json"]`
  - Specifies the file extensions (formats) to consider during auto-discovery. Defaults to "yaml".

## Automatically Added CLI Flags

These flags are automatically added to the CLI parser:

1. `--config-file <FILE>`
   - Overrides default discovery. Loads from `<FILE>` directly
2. `--no-config`
   - If set, no file is loaded. Only CLI arguments and their defaults apply
3. `--help`
   - Show help text

## Error Handling

- **Multiple Config Files:** If conflicting files (`my-tool.yaml`, `my-tool.json`) exist in the same directory, the crate exits with an error.
- **Missing Required Fields:** If a required `config_only` field is not found in the file, or if the user omits a required CLI field, an error is reported.
- **Invalid Format:** If `my-tool.yaml` is invalid YAML syntax, the crate reports a parse error.
- **No File Found:** If no file is found during walk-up and the field is required, the crate errors out (unless `--no-config` is given, in which case it's valid if the user provides enough CLI arguments).

## Configuration File Discovery

By default, if you provide `"my-tool"` as the file name using `config_file_name`:

1. The crate starts in the current directory.
2. It checks if any of `my-tool.yaml`, `my-tool.toml`, or `my-tool.json` exist (or whatever formats you specified in `config_file_formats`).
3. If not found, it walks up parent directories until it reaches the root.
4. If a file is found, it's loaded.
5. If multiple files
