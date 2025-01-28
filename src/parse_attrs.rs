use syn::{spanned::Spanned, Attribute, Lit, Meta, MetaNameValue};

/// For struct-level
#[derive(Debug, Default)]
pub struct MacroConfig {
    pub base_name: String,
    pub formats: Vec<String>,
}

/// Field-level
#[derive(Debug, Default, Clone)]
pub struct ArgAttributes {
    pub name: Option<String>,
    pub short: Option<char>,
    pub default_value: Option<String>,
    pub positional: bool,
    pub availability: FieldAvailability,
    pub multi_value_behavior: MultiValueBehavior,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FieldAvailability {
    #[default]
    Internal,
    CliOnly,
    ConfigOnly,
    CliAndConfig,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum MultiValueBehavior {
    #[default]
    Extend,
    Overwrite,
}

/// Info about each field
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub arg_attrs: ArgAttributes,
}
impl FieldInfo {
    // e.g. "bool" => is_bool_type
    pub fn is_bool_type(&self) -> bool {
        if let syn::Type::Path(tp) = &self.ty {
            if let Some(seg) = tp.path.segments.last() {
                return seg.ident == "bool";
            }
        }
        false
    }
    // e.g. "Vec<String>" => is_vec_type
    pub fn is_vec_type(&self) -> bool {
        if let syn::Type::Path(tp) = &self.ty {
            if let Some(seg) = tp.path.segments.last() {
                return seg.ident == "Vec";
            }
        }
        false
    }
}

/// Parse struct-level: #[config_file_name(...)] / #[config_file_formats(...)]
pub fn parse_struct_level_attrs(attrs: &[Attribute]) -> syn::Result<MacroConfig> {
    let mut cfg = MacroConfig::default();

    for attr in attrs {
        if let Some(ident) = attr.path().get_ident() {
            let name = ident.to_string();
            if name == "config_file_name" {
                // e.g. #[config_file_name="app-config"]
                if let Meta::NameValue(MetaNameValue {
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }),
                    ..
                }) = attr.meta.clone()
                {
                    cfg.base_name = s.value();
                }
            } else if name == "config_file_formats" {
                // e.g. #[config_file_formats="yaml,toml,json"]
                if let Meta::NameValue(MetaNameValue {
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }),
                    ..
                }) = attr.meta.clone()
                {
                    let raw = s.value();
                    // e.g. "yaml, toml, json" => ["yaml","toml","json"]
                    cfg.formats = raw.split(',').map(|x| x.trim().to_string()).collect();
                }
            }
        }
    }

    if cfg.base_name.is_empty() {
        cfg.base_name = "config".to_string();
    }
    if cfg.formats.is_empty() {
        cfg.formats = vec!["yaml".into()];
    }

    Ok(cfg)
}

/// Parse each field for #[config_arg(...)]
pub fn parse_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> syn::Result<Vec<FieldInfo>> {
    let mut out = Vec::new();
    for f in fields {
        let ident = f.ident.clone().ok_or_else(|| {
            syn::Error::new(f.span(), "Unnamed field not supported by ClapConfigFile")
        })?;

        let mut arg_attrs = ArgAttributes::default();
        let mut has_config_arg = false;

        for attr in &f.attrs {
            if let Some(ident2) = attr.path().get_ident() {
                if ident2 == "config_arg" {
                    has_config_arg = true;
                    let meta_list = attr.meta.require_list()?;
                    for nested in meta_list.parse_args_with(
                        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                    )? {
                        match nested {
                            Meta::NameValue(MetaNameValue { path, value, .. }) => {
                                let key =
                                    path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                                match (key.as_str(), value) {
                                    (
                                        "name",
                                        syn::Expr::Lit(syn::ExprLit {
                                            lit: Lit::Str(v), ..
                                        }),
                                    ) => {
                                        arg_attrs.name = Some(v.value());
                                    }
                                    (
                                        "short",
                                        syn::Expr::Lit(syn::ExprLit {
                                            lit: Lit::Char(v), ..
                                        }),
                                    ) => {
                                        arg_attrs.short = Some(v.value());
                                    }
                                    (
                                        "default_value",
                                        syn::Expr::Lit(syn::ExprLit {
                                            lit: Lit::Str(v), ..
                                        }),
                                    ) => {
                                        arg_attrs.default_value = Some(v.value());
                                    }
                                    (
                                        "accept_from",
                                        syn::Expr::Lit(syn::ExprLit {
                                            lit: Lit::Str(v), ..
                                        }),
                                    ) => {
                                        if arg_attrs.positional {
                                            return Err(syn::Error::new(
                                                attr.span(),
                                                "Positional arguments must be CLI-only. Remove accept_from attribute.",
                                            ));
                                        }
                                        match v.value().as_str() {
                                            "cli_only" => {
                                                arg_attrs.availability = FieldAvailability::CliOnly
                                            }
                                            "config_only" => {
                                                arg_attrs.availability =
                                                    FieldAvailability::ConfigOnly
                                            }
                                            "cli_and_config" => {
                                                arg_attrs.availability =
                                                    FieldAvailability::CliAndConfig
                                            }
                                            other => {
                                                return Err(syn::Error::new(
                                                    attr.span(),
                                                    format!("Invalid accept_from: {}", other),
                                                ));
                                            }
                                        }
                                    }
                                    (
                                        "multi_value_behavior",
                                        syn::Expr::Lit(syn::ExprLit {
                                            lit: Lit::Str(v), ..
                                        }),
                                    ) => match v.value().as_str() {
                                        "extend" => {
                                            arg_attrs.multi_value_behavior =
                                                MultiValueBehavior::Extend
                                        }
                                        "overwrite" => {
                                            arg_attrs.multi_value_behavior =
                                                MultiValueBehavior::Overwrite
                                        }
                                        other => {
                                            return Err(syn::Error::new(
                                                attr.span(),
                                                format!("Invalid multi_value_behavior: {}", other),
                                            ));
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            Meta::Path(path) => {
                                if let Some(kw) = path.get_ident() {
                                    if kw == "positional" {
                                        arg_attrs.positional = true;
                                        // Force positional arguments to be CLI-only
                                        arg_attrs.availability = FieldAvailability::CliOnly;
                                    } else {
                                        return Err(syn::Error::new(
                                            path.span(),
                                            format!("Unknown config_arg flag: {}", kw),
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // If no #[config_arg], mark as internal.
        // Otherwise, if user hasn't specified `accept_from`, default to "cli_and_config".
        if !has_config_arg {
            arg_attrs.availability = FieldAvailability::Internal;
        } else if arg_attrs.availability == FieldAvailability::Internal {
            arg_attrs.availability = FieldAvailability::CliAndConfig;
        }

        out.push(FieldInfo {
            ident,
            ty: f.ty.clone(),
            arg_attrs,
        });
    }
    Ok(out)
}
