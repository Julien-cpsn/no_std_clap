use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, FieldsNamed};
use crate::field::{generate_field_assignments, generate_field_parsers, parse_field_attributes};
use crate::utils::{get_inner_type, is_bool_type, is_option_type, is_vec_type};

pub fn derive_args_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;

    match input.data {
        Data::Struct(data_struct) => {
            match data_struct.fields {
                Fields::Named(fields) => {
                    let field_parsers = generate_field_parsers(&fields)?;
                    let field_assignments = generate_field_assignments(&fields)?;
                    let arg_info_generation = generate_arg_info_for_args(&fields)?;

                    let expanded = quote! {
                        impl ::no_std_clap_core::parser::Args for #name {
                            fn from_args(parsed: &::no_std_clap_core::arg::parsed_arg::ParsedArgs) -> ::core::result::Result<Self, ::no_std_clap_core::error::ParseError> {
                                use ::no_std_clap_core::arg::from_arg::FromArg;
                                use ::alloc::string::ToString;

                                #(#field_parsers)*

                                Ok(Self {
                                    #(#field_assignments)*
                                })
                            }

                            fn arg_info() -> ::alloc::vec::Vec<::no_std_clap_core::arg::arg_info::ArgInfo> {
                                use ::no_std_clap_core::arg::arg_info::ArgInfo;

                                ::alloc::vec![
                                    #(#arg_info_generation)*
                                ]
                            }

                            fn get_help(name: ::alloc::string::String, parents_name: Option<::alloc::string::String>, help: Option<::alloc::string::String>) -> ::alloc::string::String {
                                use core::fmt::Write;
                                let mut out = ::alloc::string::String::new();
                                let arg_infos = Self::arg_info();

                                if let Some(help) = help {
                                    writeln!(out, "{}", help).unwrap();
                                    writeln!(out).unwrap();
                                }

                                let name = match parents_name {
                                    Some(parents_name) => ::alloc::format!("{} {}", parents_name, name),
                                    None => name,
                                };

                                ::no_std_clap_core::help::get_help(&mut out, Some(&name), &arg_infos, &::alloc::vec::Vec::new(), &::alloc::vec::Vec::new());

                                out
                            }
                        }
                    };

                    Ok(TokenStream::from(expanded))
                }
                _ => Err(Error::new_spanned(
                    name,
                    "Args can only be derived for structs with named fields",
                )),
            }
        }
        _ => Err(Error::new_spanned(name, "Args can only be derived for structs")),
    }
}

pub fn generate_arg_definitions(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut definitions = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_attrs = parse_field_attributes(field)?;

        if field_attrs.skip {
            continue;
        }

        let field_name_str = field_name.to_string();
        let is_vec = is_vec_type(&field.ty);

        let mut arg_info_def = quote! {
            ArgInfo::new(#field_name_str)
        };

        if let Some(short) = field_attrs.short {
            let short_str = short.to_string();
            arg_info_def.extend(quote! {
                .short(#short_str.chars().next().unwrap())
            });
        }

        if let Some(long) = &field_attrs.long {
            arg_info_def.extend(quote! {
                .long(#long)
            });
        }

        if let Some(help) = &field_attrs.help {
            arg_info_def.extend(quote! {
                .help(#help)
            });
        }

        if field_attrs.required {
            arg_info_def.extend(quote! {
                .required()
            });
        }

        if field_attrs.multiple || is_vec {
            arg_info_def.extend(quote! {
                .multiple()
            });
        }

        if field_attrs.global {
            arg_info_def.extend(quote! {
                .global()
            });
        }

        if field_attrs.count {
            arg_info_def.extend(quote! {
                .count()
            });
        }

        definitions.push(arg_info_def);
    }

    Ok(definitions)
}

pub fn generate_global_arg_definitions(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut definitions = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_attrs = parse_field_attributes(field)?;

        if field_attrs.skip || !field_attrs.global {
            continue;
        }

        let field_name_str = field_name.to_string();
        let is_vec = is_vec_type(&field.ty);

        let mut arg_info_def = quote! {
            ArgInfo::new(#field_name_str)
        };

        if let Some(short) = field_attrs.short {
            let short_str = short.to_string();
            arg_info_def.extend(quote! {
                .short(#short_str.chars().next().unwrap())
            });
        }

        if let Some(long) = &field_attrs.long {
            arg_info_def.extend(quote! {
                .long(#long)
            });
        }

        if let Some(help) = &field_attrs.help {
            arg_info_def.extend(quote! {
                .help(#help)
            });
        }

        if field_attrs.required {
            arg_info_def.extend(quote! {
                .required()
            });
        }

        if field_attrs.multiple || is_vec {
            arg_info_def.extend(quote! {
                .multiple()
            });
        }

        if field_attrs.count {
            arg_info_def.extend(quote! {
                .count()
            });
        }

        // ensure .global() present for clarity (should be true here)
        arg_info_def.extend(quote! {
            .global()
        });

        definitions.push(arg_info_def);
    }

    Ok(definitions)
}


// Generate arg info for Args trait implementation
pub fn generate_arg_info_for_args(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut arg_infos = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_attrs = parse_field_attributes(field)?;

        if field_attrs.skip || field_attrs.subcommand {
            continue;
        }

        let field_name_str = field_name.to_string();
        let is_vec = is_vec_type(&field.ty);

        let mut arg_info_def = quote! {
            ArgInfo::new(#field_name_str)
        };

        if let Some(short) = field_attrs.short {
            let short_str = short.to_string();
            arg_info_def.extend(quote! {
                .short(#short_str.chars().next().unwrap())
            });
        }

        if let Some(long) = &field_attrs.long {
            arg_info_def.extend(quote! {
                .long(#long)
            });
        }

        if let Some(help) = &field_attrs.help {
            arg_info_def.extend(quote! {
                .help(#help)
            });
        }

        if field_attrs.required {
            arg_info_def.extend(quote! {
                .required()
            });
        }

        if field_attrs.multiple || is_vec {
            arg_info_def.extend(quote! {
                .multiple()
            });
        }

        if field_attrs.count {
            arg_info_def.extend(quote! {
                .count()
            });
        }

        arg_info_def.extend(quote! { , });

        arg_infos.push(arg_info_def);
    }

    Ok(arg_infos)
}

// Helper function for generating field parsers in Args context
pub fn generate_args_field_parsers(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut parsers = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_attrs = parse_field_attributes(field)?;

        if field_attrs.skip || field_attrs.subcommand {
            continue;
        }

        let field_name_str = field_name.to_string();
        let var_name = Ident::new(&format!("parsed_{}", field_name), field_name.span());

        let is_optional = is_option_type(&field.ty);
        let is_vec = is_vec_type(&field.ty);
        let is_bool = is_bool_type(&field.ty);

        let parser = if field_attrs.count {
            quote! {
                let #var_name = args.count(#field_name_str);
            }
        }
        else if is_bool {
            quote! {
                let #var_name = args.contains_key(#field_name_str);
            }
        }
        else if is_vec {
            quote! {
                let #var_name = args.get_all(#field_name_str);
            }
        }
        else if field_attrs.required && !is_optional {
            quote! {
                let #var_name = args.get(#field_name_str)
                    .ok_or_else(|| ::no_std_clap_core::error::ParseError::MissingArgument(#field_name_str.to_string()))?;
            }
        }
        else if let Some(default) = &field_attrs.default_value {
            quote! {
                let #var_name = args.get(#field_name_str)
                    .map(|s| Some(s.as_str()))
                    .unwrap_or(Some(#default));
            }
        }
        else {
            quote! {
                let #var_name = args.get(#field_name_str).map(|s| s.as_str());
            }
        };

        parsers.push(parser);
    }

    Ok(parsers)
}

// Helper function for generating field assignments in Args context
pub fn generate_args_field_assignments(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut assignments = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_attrs = parse_field_attributes(field)?;

        if field_attrs.skip || field_attrs.subcommand {
            assignments.push(quote! {
                #field_name: ::core::default::Default::default()
            });
            continue;
        }

        let var_name = Ident::new(&format!("parsed_{}", field_name), field_name.span());
        let field_type = &field.ty;

        let is_optional = is_option_type(field_type);
        let is_vec = is_vec_type(field_type);
        let is_bool = is_bool_type(field_type);
        let is_count = field_attrs.count;

        let assignment = if is_bool {
            quote! { #field_name: #var_name }
        }
        else if is_count {
            quote! {
                #field_name: #var_name.count()
            }
        }
        else if is_vec {
            let inner_type = get_inner_type(field_type).unwrap_or(field_type);
            quote! {
                #field_name: {
                    let mut vec = ::alloc::vec::Vec::new();
                    for value in #var_name {
                        vec.push(<#inner_type as ::no_std_clap_core::arg::from_arg::FromArg>::from_arg(value)?);
                    }
                    vec
                }
            }
        }
        else if is_optional {
            let inner_type = get_inner_type(field_type).unwrap_or(field_type);
            quote! {
                #field_name: match #var_name {
                    Some(s) => Some(<#inner_type as ::no_std_clap_core::arg::from_arg::FromArg>::from_arg(s)?),
                    None => None,
                }
            }
        }
        else if field_attrs.required || field_attrs.default_value.is_some() {
            quote! {
                #field_name: <#field_type as ::no_std_clap_core::arg::from_arg::FromArg>::from_arg(#var_name)?
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
            quote! {
                #field_name: match #var_name {
                    Some(s) => <#field_type as ::no_std_clap_core::arg::from_arg::FromArg>::from_arg(s)?,
                    None => ::core::default::Default::default(),
                }
            }
        };

        assignments.push(assignment);
    }

    Ok(assignments)
}
