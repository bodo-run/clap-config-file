use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Field, Fields, Lit, Meta,
    MetaNameValue,
};

/// A derive macro that enables structs to handle configuration from both CLI arguments and config files.
///
/// This macro generates the necessary code to:
/// 1. Parse command-line arguments using clap
/// 2. Load and parse config files (YAML/JSON/TOML)
/// 3. Merge the configurations with CLI taking precedence
///
/// # Attributes
/// - `#[cli_only]`: Field only appears as a CLI argument
/// - `#[config_only]`: Field only appears in config file
/// - `#[cli_and_config]`: Field appears in both (CLI overrides config)
/// - `#[config_arg(name = "x", short = 'x', long = "xxx", default_value = "y")]`: Customize CLI argument
/// - `#[multi_value_behavior(extend/overwrite)]`: For Vec fields, control merge behavior
///
/// # Example
/// ```rust
/// use clap_config_file::ClapConfigFile;
///
/// #[derive(ClapConfigFile)]
/// struct Config {
///     // CLI-only flag with custom name
///     #[cli_only]
///     #[config_arg(name = "verbose", short = 'v')]
///     verbose: bool,
///
///     // Config file only setting
///     #[config_only]
///     database_url: String,
///
///     // Available in both with CLI override
///     #[cli_and_config]
///     #[config_arg(name = "port", default_value = "8080")]
///     port: u16,
///
///     // Vec field that extends values from both sources
///     #[cli_and_config]
///     #[multi_value_behavior(extend)]
///     tags: Vec<String>,
/// }
/// ```
#[proc_macro_derive(
    ClapConfigFile,
    attributes(
        cli_only,
        config_only,
        cli_and_config,
        config_arg,
        multi_value_behavior
    )
)]
pub fn derive_clap_config_file(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match build_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Defines how a field's value can be provided, controlling whether it's available
/// via CLI arguments, config file, or both.
#[derive(Debug, Clone, Copy)]
enum FieldAvailability {
    CliOnly,
    ConfigOnly,
    CliAndConfig,
}

/// Controls how Vec fields merge values when present in both CLI and config.
/// - Extend: Append CLI values to config values
/// - Overwrite: Replace config values with CLI values if present
#[derive(Debug, Clone, Default)]
enum MultiValueBehavior {
    #[default]
    Extend,
    Overwrite,
}

/// Holds CLI argument customization options specified via #[config_arg(...)]
#[derive(Debug, Default, Clone)]
struct ArgAttributes {
    name: Option<String>,
    short: Option<char>,
    long: Option<String>,
    default_value: Option<String>,
}

/// Aggregates all metadata about a struct field needed for code generation
#[derive(Debug, Clone)]
struct FieldInfo {
    availability: FieldAvailability,
    multi_value_behavior: MultiValueBehavior,
    arg_attrs: ArgAttributes,
    ident: syn::Ident,
    ty: syn::Type,
}

/// Main implementation function that generates the expanded code for the derive macro.
/// This includes:
/// - Hidden CLI struct with clap attributes
/// - Hidden config struct with serde attributes
/// - Implementation of parsing and merging logic
/// - Helper functions for config file discovery and loading
fn build_impl(ast: &DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = &ast.ident;
    let generics = &ast.generics;

    // We only handle a struct with named fields
    let fields = match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => named.named.iter().collect::<Vec<_>>(),
            _ => {
                return Err(syn::Error::new_spanned(
                    &ast.ident,
                    "Only structs with named fields are supported.",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &ast.ident,
                "Only structs with named fields are supported.",
            ))
        }
    };

    // Parse user's field attributes
    let parsed_fields: Vec<FieldInfo> = fields
        .iter()
        .map(|field| parse_one_field(field))
        .collect::<Result<_, _>>()?;

    // For each field, generate the CLI struct token, the config struct token, and the final merge expression
    let mut cli_struct_fields = Vec::new();
    let mut cfg_struct_fields = Vec::new();
    let mut merge_stmts = Vec::new();

    for pf in &parsed_fields {
        // Generate the appropriate tokens
        let cli_ts_opt = generate_cli_field_tokens(pf);
        let cfg_ts_opt = generate_config_field_tokens(pf);
        let merge_expr = generate_merge_expr(pf);

        // If we got Some(...) token stream, push it into the vectors
        if let Some(ts) = cli_ts_opt {
            cli_struct_fields.push(ts);
        }
        if let Some(ts) = cfg_ts_opt {
            cfg_struct_fields.push(ts);
        }

        // Always push the final merge line (the user's struct field)
        let field_name = &pf.ident;
        merge_stmts.push(quote! {
            #field_name: #merge_expr
        });
    }

    // Add our special meta fields (for --no-config, etc.) to the CLI struct
    cli_struct_fields.push(quote! {
        /// If true, skip reading any config file
        #[arg(long = "no-config", default_value_t = false)]
        pub __no_config: bool,

        /// Explicit config-file path
        #[arg(long = "config-file")]
        pub __config_file: Option<std::path::PathBuf>,

        /// Optional raw config string in JSON/YAML/TOML
        #[arg(long = "config")]
        pub __raw_config: Option<String>,
    });

    // Construct hidden struct names
    let cli_struct_ident = syn::Ident::new(&format!("__{}_Cli", struct_name), Span::call_site());
    let cfg_struct_ident = syn::Ident::new(&format!("__{}_Cfg", struct_name), Span::call_site());

    let num_fields = parsed_fields.len();
    let field_names = parsed_fields.iter().map(|pf| &pf.ident);

    // Build final output
    let expanded = quote! {
        // Hidden CLI struct
        #[derive(::clap::Parser, Debug, Default)]
        struct #cli_struct_ident {
            #(#cli_struct_fields),*
        }

        // Hidden config struct
        #[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
        struct #cfg_struct_ident {
            #(#cfg_struct_fields),*
        }

        impl #struct_name #generics {
            pub fn parse_with_default_file_name(default_file_name: &str) -> Self {
                use ::clap::Parser;
                let cli = #cli_struct_ident::parse();

                if cli.__no_config {
                    let cfg = #cfg_struct_ident::default();
                    return Self::from_parts(cli, cfg);
                }

                let path = if let Some(ref p) = cli.__config_file {
                    Some(p.clone())
                } else {
                    find_config_by_walking_up(default_file_name)
                };

                let mut loaded_cfg = match load_config_file(path.as_ref()) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        if let Some(ref path) = path {
                            eprintln!("Warning: could not load config file {}: {}", path.display(), e);
                        }
                        #cfg_struct_ident::default()
                    }
                };

                if let Some(ref raw) = cli.__raw_config {
                    if let Ok(extra) = parse_raw_config(raw) {
                        loaded_cfg = merge_configs(loaded_cfg, extra);
                    } else {
                        eprintln!("Warning: failed to parse raw config from --config");
                    }
                }

                Self::from_parts(cli, loaded_cfg)
            }

            pub fn parse() -> Self {
                Self::parse_with_default_file_name("config.yaml")
            }

            pub fn parse_from<I, T>(iter: I) -> Self
            where
                I: IntoIterator<Item = T>,
                T: Into<std::ffi::OsString> + Clone,
            {
                use ::clap::Parser;
                let cli = #cli_struct_ident::parse_from(iter);
                let cfg = #cfg_struct_ident::default();
                Self::from_parts(cli, cfg)
            }

            fn from_parts(cli: #cli_struct_ident, cfg: #cfg_struct_ident) -> Self {
                Self {
                    #(#merge_stmts),*
                }
            }
        }

        // Implement Serialize for the final user struct
        impl #generics serde::Serialize for #struct_name #generics {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                use serde::ser::SerializeStruct;
                let mut st = serializer.serialize_struct(stringify!(#struct_name), #num_fields)?;
                #( st.serialize_field(stringify!(#field_names), &self.#field_names)?; )*
                st.end()
            }
        }

        fn find_config_by_walking_up(file_name: &str) -> Option<std::path::PathBuf> {
            let mut dir = std::env::current_dir().ok()?;
            loop {
                let candidate = dir.join(file_name);
                if candidate.is_file() {
                    if let Some(conflict) = has_conflicting_configs(&dir, file_name) {
                        eprintln!(
                            "Error: Multiple config files found ({:?} and {:?}). \
                             Use --config-file to pick one.",
                            candidate.file_name().unwrap_or_default(),
                            conflict.file_name().unwrap_or_default()
                        );
                        std::process::exit(2);
                    }
                    return Some(candidate);
                }
                if !dir.pop() {
                    break;
                }
            }
            None
        }

        fn has_conflicting_configs(dir: &std::path::Path, file_name: &str) -> Option<std::path::PathBuf> {
            let known_exts = ["yaml", "yml", "json", "toml"];
            let base_name = std::path::Path::new(file_name)
                .file_stem()
                .map(|os| os.to_string_lossy().to_string())?;
            let expected_ext = std::path::Path::new(file_name)
                .extension()
                .map(|os| os.to_string_lossy().to_string())
                .unwrap_or_default();

            let entries = std::fs::read_dir(dir).ok()?;
            for e in entries.flatten() {
                let path = e.path();
                if path.is_file() {
                    if let Some(stem) = path.file_stem() {
                        if stem.to_string_lossy() == base_name {
                            if let Some(ext) = path.extension() {
                                let ext_s = ext.to_string_lossy();
                                if ext_s != expected_ext && known_exts.contains(&ext_s.as_ref()) {
                                    return Some(path);
                                }
                            }
                        }
                    }
                }
            }
            None
        }

        fn load_config_file(path: Option<&std::path::PathBuf>) -> Result<#cfg_struct_ident, Box<dyn std::error::Error>> {
            if let Some(p) = path {
                let content = std::fs::read_to_string(p)?;
                let ext = p.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                return match ext.as_str() {
                    "json" => Ok(serde_json::from_str(&content)?),
                    "toml" => Ok(toml::from_str(&content)?),
                    _      => Ok(serde_yaml::from_str(&content)?), // default to YAML
                };
            }
            Ok(#cfg_struct_ident::default())
        }

        fn parse_raw_config(raw: &str) -> Result<#cfg_struct_ident, Box<dyn std::error::Error>> {
            // Try JSON -> YAML -> TOML
            if let Ok(val) = serde_json::from_str(raw) {
                return Ok(val);
            }
            if let Ok(val) = serde_yaml::from_str(raw) {
                return Ok(val);
            }
            if let Ok(val) = toml::from_str(raw) {
                return Ok(val);
            }
            Err("Cannot parse --config <RAW> as JSON, YAML, or TOML".into())
        }

        fn merge_configs(mut base: #cfg_struct_ident, other: #cfg_struct_ident) -> #cfg_struct_ident {
            let base_json = match serde_json::to_value(&base) {
                Ok(val) => val,
                Err(_) => return base,
            };
            let other_json = match serde_json::to_value(&other) {
                Ok(val) => val,
                Err(_) => return base,
            };
            let merged = deep_merge_json(base_json, other_json);
            match serde_json::from_value(merged) {
                Ok(val) => val,
                Err(_) => base,
            }
        }

        fn deep_merge_json(base: serde_json::Value, over: serde_json::Value) -> serde_json::Value {
            match (base, over) {
                (serde_json::Value::Object(mut b), serde_json::Value::Object(o)) => {
                    for (k, v) in o {
                        if !v.is_null() {
                            let old = b.remove(&k).unwrap_or(serde_json::Value::Null);
                            b.insert(k, deep_merge_json(old, v));
                        }
                    }
                    serde_json::Value::Object(b)
                }
                (_, over_any) => over_any,
            }
        }
    };

    Ok(expanded)
}

/// Parses all attributes on a field to determine its configuration behavior
fn parse_one_field(field: &Field) -> syn::Result<FieldInfo> {
    let mut availability = None;
    let mut mv_behavior = MultiValueBehavior::default();
    let mut arg_attrs = ArgAttributes::default();

    // Check each attribute for relevant markers
    for attr in &field.attrs {
        let path_ident = match attr.path().get_ident() {
            Some(i) => i.to_string(),
            None => continue,
        };

        match path_ident.as_str() {
            "cli_only" => {
                ensure_avail_none(&availability, attr)?;
                availability = Some(FieldAvailability::CliOnly);
            }
            "config_only" => {
                ensure_avail_none(&availability, attr)?;
                availability = Some(FieldAvailability::ConfigOnly);
            }
            "cli_and_config" => {
                ensure_avail_none(&availability, attr)?;
                availability = Some(FieldAvailability::CliAndConfig);
            }
            "config_arg" => {
                let meta = attr.meta.require_list()?;
                for nested in meta.parse_args_with(
                    syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                )? {
                    match nested {
                        Meta::Path(_) => {}
                        Meta::NameValue(MetaNameValue { path, value, .. }) => {
                            let key = path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                            if let syn::Expr::Lit(l) = value {
                                match l.lit {
                                    Lit::Str(sval) => match key.as_str() {
                                        "default_value" => {
                                            arg_attrs.default_value = Some(sval.value());
                                        }
                                        "long" => {
                                            arg_attrs.long = Some(sval.value());
                                        }
                                        "name" => {
                                            // treat "name" as "long"
                                            arg_attrs.long = Some(sval.value());
                                        }
                                        _ => {}
                                    },
                                    Lit::Char(cval) => {
                                        if key == "short" {
                                            arg_attrs.short = Some(cval.value());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Meta::List(_) => {}
                    }
                }
            }
            "multi_value_behavior" => {
                let meta = attr.meta.require_list()?;
                for nested in meta.parse_args_with(
                    syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                )? {
                    if let Meta::NameValue(MetaNameValue {
                        value:
                            syn::Expr::Lit(syn::ExprLit {
                                lit: Lit::Str(s), ..
                            }),
                        ..
                    }) = nested
                    {
                        match s.value().as_str() {
                            "extend" => mv_behavior = MultiValueBehavior::Extend,
                            "overwrite" => mv_behavior = MultiValueBehavior::Overwrite,
                            other => {
                                return Err(syn::Error::new_spanned(
                                    attr,
                                    format!("Invalid multi_value_behavior: {}", other),
                                ))
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let final_avail = availability.unwrap_or(FieldAvailability::CliAndConfig);

    Ok(FieldInfo {
        availability: final_avail,
        multi_value_behavior: mv_behavior,
        arg_attrs,
        ident: field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new(field.span(), "Field must be named"))?,
        ty: field.ty.clone(),
    })
}

/// Validates that only one availability attribute is specified per field
fn ensure_avail_none(avail: &Option<FieldAvailability>, attr: &Attribute) -> syn::Result<()> {
    if avail.is_some() {
        Err(syn::Error::new_spanned(
            attr,
            "Only one of [cli_only, config_only, cli_and_config] is allowed per field",
        ))
    } else {
        Ok(())
    }
}

/// Generates the field definition for the hidden CLI struct.
/// Handles:
/// - Optional wrapping for all fields
/// - Special handling for bool fields
/// - CLI argument customization
fn generate_cli_field_tokens(pf: &FieldInfo) -> Option<TokenStream2> {
    match pf.availability {
        FieldAvailability::ConfigOnly => None,
        FieldAvailability::CliOnly | FieldAvailability::CliAndConfig => {
            let field_name = &pf.ident;
            let ty = &pf.ty;
            let ArgAttributes {
                name,
                short,
                long,
                default_value,
            } = &pf.arg_attrs;

            // Create owned String here instead of referencing temporary
            let name_str = name.clone().unwrap_or_else(|| field_name.to_string());

            let short_attr = short.map(|c| quote!(short = #c,));
            let long_attr = if let Some(ref l) = long {
                quote!(long = #l,)
            } else {
                quote!(long = #name_str,)
            };

            let default_attr = if let Some(dv) = default_value {
                quote!(default_value = #dv,)
            } else {
                quote!()
            };

            // If it's bool, use Option<bool> + ArgAction::SetTrue
            let (final_ty, action) = if is_bool_type(ty) {
                (
                    quote!(Option<bool>),
                    quote!(action = ::clap::ArgAction::SetTrue,),
                )
            } else {
                (quote!(Option<#ty>), quote!())
            };

            Some(quote! {
                #[arg(#short_attr #long_attr #default_attr #action)]
                pub #field_name: #final_ty
            })
        }
    }
}

/// Generates the field definition for the hidden config struct.
/// All fields are marked with #[serde(default)] to handle missing values.
fn generate_config_field_tokens(pf: &FieldInfo) -> Option<TokenStream2> {
    match pf.availability {
        FieldAvailability::CliOnly => {
            // No config field for CLI-only
            None
        }
        FieldAvailability::ConfigOnly | FieldAvailability::CliAndConfig => {
            let field_name = &pf.ident;
            let ty = &pf.ty;
            // We just do `#[serde(default)] pub field_name: T`
            Some(quote! {
                #[serde(default)]
                pub #field_name: #ty
            })
        }
    }
}

/// Generates the expression that combines CLI and config values for a field.
/// The merge strategy depends on:
/// - Field availability (cli_only, config_only, cli_and_config)
/// - Whether the field is a Vec (uses multi_value_behavior)
/// - For non-Vec fields, CLI takes precedence if present
fn generate_merge_expr(pf: &FieldInfo) -> TokenStream2 {
    let field_name = &pf.ident;
    match pf.availability {
        FieldAvailability::CliOnly => {
            // final = cli.field.unwrap_or_default()
            quote! {
                cli.#field_name.unwrap_or_default()
            }
        }
        FieldAvailability::ConfigOnly => {
            // final = cfg.field
            quote! {
                cfg.#field_name
            }
        }
        FieldAvailability::CliAndConfig => {
            let is_vec = is_vec_type(&pf.ty);
            if is_vec {
                match pf.multi_value_behavior {
                    MultiValueBehavior::Extend => quote! {
                        {
                            let mut merged = cfg.#field_name;
                            if let Some(cli_vec) = cli.#field_name {
                                merged.extend(cli_vec);
                            }
                            merged
                        }
                    },
                    MultiValueBehavior::Overwrite => quote! {
                        if let Some(cli_vec) = cli.#field_name {
                            cli_vec
                        } else {
                            cfg.#field_name
                        }
                    },
                }
            } else {
                // normal scalar
                quote! {
                    cli.#field_name.unwrap_or(cfg.#field_name)
                }
            }
        }
    }
}

/// Determines if a type is a bool by checking its path segments
fn is_bool_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident == "bool";
        }
    }
    false
}

/// Determines if a type is a Vec by checking its path segments
fn is_vec_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            if seg.ident == "Vec" {
                return true;
            }
        }
    }
    false
}
