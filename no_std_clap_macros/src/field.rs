use crate::utils::{get_inner_type, is_bool_type, is_option_type, is_vec_type};
use quote::{format_ident, quote};
use syn::{Error, Expr, Field, FieldsNamed, LitStr, Meta};

#[derive(Default)]
pub struct FieldAttributes {
    pub short: Option<char>,
    pub long: Option<String>,
    pub help: Option<String>,
    pub required: bool,
    pub multiple: bool,
    pub default_value: Option<Expr>,
    pub skip: bool,
    pub subcommand: bool,
    pub global: bool,
}

pub fn parse_field_attributes(field: &Field) -> Result<FieldAttributes, Error> {
    let mut field_attrs = FieldAttributes::default();

    for attr in &field.attrs {
        if attr.path().is_ident("arg") {
            match &attr.meta {
                Meta::List(_) => {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("short") {
                            if let Ok(value) = meta.value() {
                                let lit: LitStr = value.parse()?;
                                let s = lit.value();
                                if s.len() == 1 {
                                    field_attrs.short = s.chars().next();
                                }
                                else {
                                    return Err(meta.error("short must be a single character"));
                                }
                            }
                            else {
                                // Infer short from field name
                                if let Some(ident) = &field.ident {
                                    field_attrs.short = ident.to_string().chars().next();
                                }
                            }
                        }
                        else if meta.path.is_ident("long") {
                            if let Ok(value) = meta.value() {
                                let lit: LitStr = value.parse()?;
                                field_attrs.long = Some(lit.value());
                            }
                            else {
                                // Infer long from field name
                                if let Some(ident) = &field.ident {
                                    field_attrs.long = Some(ident.to_string().replace('_', "-"));
                                }
                            }
                        }
                        else if meta.path.is_ident("help") {
                            let value: LitStr = meta.value()?.parse()?;
                            field_attrs.help = Some(value.value());
                        }
                        else if meta.path.is_ident("required") {
                            field_attrs.required = true;
                        }
                        else if meta.path.is_ident("multiple") {
                            field_attrs.multiple = true;
                        }
                        else if meta.path.is_ident("default_value") {
                            let value: Expr = meta.value()?.parse()?;
                            field_attrs.default_value = Some(value);
                        }
                        else if meta.path.is_ident("skip") {
                            field_attrs.skip = true;
                        }
                        else if meta.path.is_ident("global") {
                            field_attrs.global = true;
                        }
                        Ok(())
                    })?;
                }
                Meta::Path(_) => {
                    // #[arg] without parameters - infer from field name
                    if let Some(ident) = &field.ident {
                        let field_name = ident.to_string();
                        field_attrs.short = field_name.chars().next();
                        field_attrs.long = Some(field_name.replace('_', "-"));
                    }
                }
                _ => {}
            }
        }
        else if attr.path().is_ident("command") {
            match &attr.meta {
                Meta::List(_) => {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("subcommand") {
                            field_attrs.subcommand = true;
                        }
                        Ok(())
                    })?;
                }
                Meta::Path(_) => {
                    field_attrs.subcommand = true;
                }
                _ => {}
            }
        }
    }

    if is_vec_type(&field.ty) && !field_attrs.multiple {
        field_attrs.multiple = true;
    }

    Ok(field_attrs)
}

pub fn generate_field_parsers(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut parsers = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_attrs = parse_field_attributes(field)?;

        if field_attrs.skip {
            continue;
        }

        let field_name_str = field_name.to_string();
        let var_name = format_ident!("parsed_{}", field_name);

        if field_attrs.subcommand {
            // This is a subcommand field
            let parser = quote! {
                let #var_name = parsed.subcommand.as_ref();
            };
            parsers.push(parser);
        }
        else {
            let is_optional = is_option_type(&field.ty);
            let is_vec = is_vec_type(&field.ty);
            let is_bool = is_bool_type(&field.ty);

            let parser = if is_bool {
                // Boolean flags don't take values
                quote! {
                    let #var_name = parsed.contains_key(#field_name_str);
                }
            }
            else if is_vec {
                // Vec types can have multiple values
                quote! {
                    let #var_name = parsed.get_all(#field_name_str);
                }
            }
            else if field_attrs.required && !is_optional {
                quote! {
                    let #var_name = parsed.get(#field_name_str)
                        .ok_or_else(|| ::no_std_clap_core::error::ParseError::MissingArgument(#field_name_str.to_string()))?;
                }
            }
            else if let Some(default) = &field_attrs.default_value {
                if is_optional {
                    quote! {
                        let #var_name = parsed.get(#field_name_str)
                            .map(|s| Some(s.as_str()))
                            .unwrap_or(Some(#default));
                    }
                }
                else {
                    quote! {
                    let #var_name = parsed.get(#field_name_str)
                        .map(|s| s.as_str())
                        .unwrap_or(#default);
                }
                }
            }
            else {
                quote! {
                    let #var_name = parsed.get(#field_name_str).map(|s| s.as_str());
                }
            };

            parsers.push(parser);
        }
    }

    Ok(parsers)
}

pub fn generate_field_assignments(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut assignments = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_attrs = parse_field_attributes(field)?;

        if field_attrs.skip {
            // Use Default::default() for skipped fields
            assignments.push(quote! {
                #field_name: ::core::default::Default::default()
            });
            continue;
        }

        let var_name = format_ident!("parsed_{}", field_name);
        let field_type = &field.ty;

        let is_optional = is_option_type(field_type);

        if field_attrs.subcommand {
            let assignment = if is_optional {
                quote! {
                    #field_name: match #var_name {
                        Some((name, args)) => <#field_type as Subcommand>::from_subcommand(name, args)?,
                        None => None,
                    },
                }
            } else {
                quote! {
                    #field_name: {
                        let (name, args) = #var_name.ok_or(::no_std_clap_core::error::ParseError::MissingSubcommand)?;
                        <#field_type as Subcommand>::from_subcommand(name, args)?
                    },
                }
            };

            assignments.push(assignment);
        }
        else {
            let is_vec = is_vec_type(field_type);
            let is_bool = is_bool_type(field_type);

            let mut assignment = if is_bool {
                quote! {
                    #field_name: #var_name
                }
            }
            else if is_vec {
                // For Vec<T>, parse each value and collect into a vector
                let inner_type = get_inner_type(field_type).unwrap_or(field_type);
                quote! {
                    #field_name: {
                        let mut vec = ::alloc::vec::Vec::new();
                        for value in #var_name {
                            vec.push(<#inner_type as FromArg>::from_arg(value)?);
                        }
                        vec
                    }
                }
            }
            else if is_optional {
                // For Option<T>, we need to get the inner type T
                let inner_type = get_inner_type(field_type).unwrap_or(field_type);
                quote! {
                    #field_name: match #var_name {
                        Some(s) => Some(<#inner_type as FromArg>::from_arg(s)?),
                        None => None,
                    }
                }
            }
            else if field_attrs.required || field_attrs.default_value.is_some() {
                quote! {
                    #field_name: <#field_type as FromArg>::from_arg(#var_name)?
                }
            }
            else if is_optional {
                let inner_type = get_inner_type(field_type).unwrap_or(field_type);
                quote! {
                    #field_name: match #var_name {
                        Some(s) => Some(<#inner_type as FromArg>::from_arg(s)?),
                        None => None,
                    }
                }
            }
            else {
                // Non-optional, non-required, no default -> error at runtime
                quote! {
                    #field_name: {
                        let s = #var_name.ok_or_else(||
                            ::no_std_clap_core::error::ParseError::MissingArgument(
                                stringify!(#field_name).to_string()
                            )
                        )?;
                        <#field_type as FromArg>::from_arg(s)?
                    }
                }
            };

            assignment.extend(quote! { , });

            assignments.push(assignment);
        }
    }

    Ok(assignments)
}
