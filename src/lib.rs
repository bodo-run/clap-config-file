//! A single-derive macro merging Clap + config, defaulting field names to kebab-case.
//! Now supports bool fields with or without default_value, avoiding parse errors.

use heck::ToKebabCase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error, LitStr};

mod parse_attrs;
use parse_attrs::*;

#[proc_macro_derive(
    ClapConfigFile,
    attributes(config_file_name, config_file_formats, config_arg)
)]
pub fn derive_clap_config_file(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match build_impl(ast) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn build_impl(ast: DeriveInput) -> syn::Result<TokenStream2> {
    let struct_ident = &ast.ident;
    let generics = &ast.generics;

    let macro_cfg = parse_struct_level_attrs(&ast.attrs)?;

    let fields_named = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref named),
            ..
        }) => &named.named,
        _ => {
            return Err(Error::new_spanned(
                &ast.ident,
                "ClapConfigFile only supports a struct with named fields.",
            ))
        }
    };

    let field_infos = parse_fields(fields_named)?;
    let parse_info_impl = generate_parse_info_impl(struct_ident, &field_infos, &macro_cfg);

    let debug_impl = generate_debug_impl(struct_ident, generics, &field_infos);
    let serialize_impl = generate_serialize_impl(struct_ident, generics, &field_infos);

    let expanded = quote! {
        impl #generics #struct_ident #generics {
            pub fn parse_info() -> (Self, Option<std::path::PathBuf>, Option<&'static str>) {
                #parse_info_impl
            }
            pub fn parse() -> Self {
                Self::parse_info().0
            }
        }

        #debug_impl
        #serialize_impl
    };

    Ok(expanded)
}

/// Generate parse_info: ephemeral CLI + ephemeral config => unify.
fn generate_parse_info_impl(
    struct_ident: &syn::Ident,
    fields: &[FieldInfo],
    macro_cfg: &MacroConfig,
) -> TokenStream2 {
    let base_name = &macro_cfg.base_name;
    let fmts = &macro_cfg.formats;
    let fmts_list: Vec<_> = fmts.iter().map(|s| s.as_str()).collect();

    // ephemeral CLI
    let cli_ident = syn::Ident::new(&format!("__{}_Cli", struct_ident), Span::call_site());
    let cli_fields = fields
        .iter()
        .filter(|f| {
            !matches!(
                f.arg_attrs.availability,
                FieldAvailability::ConfigOnly | FieldAvailability::Internal
            )
        })
        .map(generate_cli_field);

    let cli_extras = quote! {
        #[clap(long="no-config", default_value_t=false, help="Do not use a config file")]
        __no_config: bool,

        #[clap(long="config-file", help="Path to the config file")]
        __config_file: Option<std::path::PathBuf>,
    };
    let build_cli_struct = quote! {
        #[derive(::clap::Parser, ::std::fmt::Debug, ::std::default::Default)]
        struct #cli_ident {
            #cli_extras
            #(#cli_fields),*
        }
    };

    // ephemeral config
    let cfg_ident = syn::Ident::new(&format!("__{}_Cfg", struct_ident), Span::call_site());
    let cfg_fields = fields
        .iter()
        .filter(|f| {
            !matches!(
                f.arg_attrs.availability,
                FieldAvailability::CliOnly | FieldAvailability::Internal
            )
        })
        .map(generate_config_field);
    let build_cfg_struct = quote! {
        #[derive(::serde::Deserialize, ::std::fmt::Debug, ::std::default::Default)]
        struct #cfg_ident {
            #(#cfg_fields),*
        }
    };

    let unify_stmts = fields.iter().map(unify_field);

    let inline_helpers = quote! {
        fn __inline_guess_format(path: &std::path::Path, known_formats: &[&str]) -> Option<&'static str> {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
                for &f in known_formats {
                    if ext == f {
                        return Some(Box::leak(f.to_string().into_boxed_str()));
                    }
                }
            }
            None
        }

        fn __inline_find_config(base_name: &str, fmts: &[&str]) -> Option<std::path::PathBuf> {
            let mut dir = std::env::current_dir().ok()?;
            let mut found: Option<std::path::PathBuf> = None;

            loop {
                let mut found_this = vec![];
                for &f in fmts {
                    let candidate = dir.join(format!("{}.{}", base_name, f));
                    if candidate.is_file() {
                        found_this.push(candidate);
                    }
                }
                if found_this.len() > 1 {
                    eprintln!("Error: multiple config files in same dir: {:?}", found_this);
                    std::process::exit(2);
                } else if found_this.len() == 1 {
                    if found.is_some() {
                        eprintln!(
                            "Error: multiple config files found walking up: {:?} and {:?}",
                            found.as_ref().unwrap(), found_this[0]
                        );
                        std::process::exit(2);
                    }
                    found = Some(found_this.remove(0));
                }
                if !dir.pop() {
                    break;
                }
            }
            found
        }
    };

    quote! {
        #build_cli_struct
        #build_cfg_struct

        use ::clap::Parser;
        let cli = #cli_ident::parse();

        #inline_helpers

        let mut used_path: Option<std::path::PathBuf> = None;
        let mut used_format: Option<&'static str> = None;

        let mut config_data = ::config::Config::builder();
        if !cli.__no_config {
            if let Some(ref path) = cli.__config_file {
                used_path = Some(path.clone());
                let format = __inline_guess_format(path, &[#(#fmts_list),*]);
                if let Some(fmt) = format {
                    let file = match fmt {
                        "yaml" | "yml" => ::config::File::from(path.as_path()).format(::config::FileFormat::Yaml),
                        "json" => ::config::File::from(path.as_path()).format(::config::FileFormat::Json),
                        "toml" => ::config::File::from(path.as_path()).format(::config::FileFormat::Toml),
                        _ => ::config::File::from(path.as_path()).format(::config::FileFormat::Yaml),
                    };
                    config_data = config_data.add_source(file);
                }
                used_format = format;
            } else if let Some(found) = __inline_find_config(#base_name, &[#(#fmts_list),*]) {
                used_path = Some(found.clone());
                let format = __inline_guess_format(&found, &[#(#fmts_list),*]);
                if let Some(fmt) = format {
                    let file = match fmt {
                        "yaml" | "yml" => ::config::File::from(found.as_path()).format(::config::FileFormat::Yaml),
                        "json" => ::config::File::from(found.as_path()).format(::config::FileFormat::Json),
                        "toml" => ::config::File::from(found.as_path()).format(::config::FileFormat::Toml),
                        _ => ::config::File::from(found.as_path()).format(::config::FileFormat::Yaml),
                    };
                    config_data = config_data.add_source(file);
                }
                used_format = format;
            }
        }

        let built = config_data.build().unwrap_or_else(|e| {
            eprintln!("Failed to build config: {}", e);
            ::config::Config::default()
        });
        let ephemeral_cfg: #cfg_ident = built.clone().try_deserialize().unwrap_or_else(|e| {
            eprintln!("Failed to deserialize config into struct: {}", e);
            eprintln!("Config data after build: {:#?}", built);
            #cfg_ident::default()
        });


        let final_struct = #struct_ident {
            #(#unify_stmts),*
        };
        (final_struct, used_path, used_format)
    }
}

/// Generate ephemeral CLI field if field is not config_only
fn generate_cli_field(field: &FieldInfo) -> TokenStream2 {
    let ident = &field.ident;
    let kebab_default = ident.to_string().to_kebab_case();
    let final_name = field.arg_attrs.name.clone().unwrap_or(kebab_default);
    let name_lit = LitStr::new(&final_name, Span::call_site());
    let help_text = &field.arg_attrs.help_text;
    let help_attr = if help_text.is_empty() {
        quote!()
    } else {
        let help_lit = LitStr::new(help_text, Span::call_site());
        quote!(help=#help_lit,)
    };

    if field.arg_attrs.positional {
        // For positional arguments
        if field.is_vec_type() {
            quote! {
                #[clap(value_name=#name_lit, num_args=1.., action=::clap::ArgAction::Append, #help_attr)]
                #ident: Option<Vec<String>>
            }
        } else {
            quote! {
                #[clap(value_name=#name_lit, #help_attr)]
                #ident: Option<String>
            }
        }
    } else {
        // short?
        let short_attr = if let Some(ch) = field.arg_attrs.short {
            quote!(short=#ch,)
        } else {
            quote!()
        };

        if field.is_bool_type() {
            // Handle bool default_value "true"/"false"
            if let Some(ref dv) = field.arg_attrs.default_value {
                let is_true = dv.eq_ignore_ascii_case("true");
                let is_false = dv.eq_ignore_ascii_case("false");
                if !is_true && !is_false {
                    let msg = format!(
                        "For bool field, default_value must be \"true\" or \"false\", got {}",
                        dv
                    );
                    return quote! {
                        compile_error!(#msg);
                        #ident: ()
                    };
                }
                let bool_lit = if is_true { quote!(true) } else { quote!(false) };
                quote! {
                    #[clap(long=#name_lit, #short_attr default_value_t=#bool_lit, #help_attr)]
                    #ident: Option<bool>
                }
            } else {
                quote! {
                    #[clap(long=#name_lit, #short_attr action=::clap::ArgAction::SetTrue, #help_attr)]
                    #ident: Option<bool>
                }
            }
        } else {
            let dv_attr = if let Some(dv) = &field.arg_attrs.default_value {
                let dv_lit = LitStr::new(dv, Span::call_site());
                quote!(default_value=#dv_lit,)
            } else {
                quote!()
            };
            let is_vec = field.is_vec_type();
            let multi = if is_vec {
                quote!(num_args = 1.., action = ::clap::ArgAction::Append,)
            } else {
                quote!()
            };
            let field_ty = {
                let t = &field.ty;
                quote!(Option<#t>)
            };

            quote! {
                #[clap(long=#name_lit, #short_attr #dv_attr #multi #help_attr)]
                #ident: #field_ty
            }
        }
    }
}
/// Generate ephemeral config field if field is not cli_only
fn generate_config_field(field: &FieldInfo) -> TokenStream2 {
    let ident = &field.ident;
    let ty = &field.ty;

    // Only use rename if explicitly specified
    let rename_attr = if let Some(name) = &field.arg_attrs.name {
        let name_lit = LitStr::new(name, Span::call_site());
        quote!(#[serde(rename = #name_lit)])
    } else {
        quote!()
    };

    quote! {
        #rename_attr
        #[serde(default)]
        pub #ident: #ty
    }
}

/// Merge ephemeral CLI + ephemeral config => final
fn unify_field(field: &FieldInfo) -> TokenStream2 {
    let ident = &field.ident;
    match field.arg_attrs.availability {
        FieldAvailability::CliOnly => {
            if field.is_vec_type() {
                quote!(#ident: cli.#ident.unwrap_or_default())
            } else if field.is_bool_type() {
                quote!(#ident: cli.#ident.unwrap_or(false))
            } else {
                quote!(#ident: cli.#ident.unwrap_or_default())
            }
        }
        FieldAvailability::ConfigOnly => {
            quote!(#ident: ephemeral_cfg.#ident)
        }
        FieldAvailability::CliAndConfig => {
            if field.is_vec_type() {
                match field.arg_attrs.multi_value_behavior {
                    MultiValueBehavior::Extend => quote! {
                        #ident: {
                            let mut merged = ephemeral_cfg.#ident.clone();
                            if let Some(cli_vec) = cli.#ident {
                                merged.extend(cli_vec);
                            }
                            merged
                        }
                    },
                    MultiValueBehavior::Overwrite => quote! {
                        #ident: cli.#ident.unwrap_or_else(|| ephemeral_cfg.#ident.clone())
                    },
                }
            } else if field.is_bool_type() {
                quote!(#ident: cli.#ident.unwrap_or(ephemeral_cfg.#ident))
            } else {
                quote!(#ident: cli.#ident.unwrap_or_else(|| ephemeral_cfg.#ident))
            }
        }
        FieldAvailability::Internal => {
            quote!(#ident: Default::default())
        }
    }
}

/// Implement Debug for final struct
fn generate_debug_impl(
    struct_ident: &syn::Ident,
    generics: &syn::Generics,
    fields: &[FieldInfo],
) -> TokenStream2 {
    let field_idents = fields.iter().map(|fi| &fi.ident);
    quote! {
        impl #generics ::std::fmt::Debug for #struct_ident #generics {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let mut dbg = f.debug_struct(stringify!(#struct_ident));
                #( dbg.field(stringify!(#field_idents), &self.#field_idents); )*
                dbg.finish()
            }
        }
    }
}

/// Implement Serialize for final struct
fn generate_serialize_impl(
    struct_ident: &syn::Ident,
    generics: &syn::Generics,
    fields: &[FieldInfo],
) -> TokenStream2 {
    let field_idents = fields.iter().map(|fi| &fi.ident);
    let field_names = fields.iter().map(|fi| fi.ident.to_string());
    let num_fields = fields.len();

    quote! {
        impl #generics ::serde::Serialize for #struct_ident #generics {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer
            {
                use ::serde::ser::SerializeStruct;
                let mut st = serializer.serialize_struct(
                    stringify!(#struct_ident),
                    #num_fields
                )?;
                #(
                    st.serialize_field(#field_names, &self.#field_idents)?;
                )*
                st.end()
            }
        }
    }
}
