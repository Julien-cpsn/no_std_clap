use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Error, Expr, Fields, Lit, LitStr, Meta};
use crate::args::{generate_arg_definitions, generate_global_arg_definitions};
use crate::field::{generate_field_assignments, generate_field_parsers};
use crate::subcommand::generate_subcommand_definitions;
use crate::utils::to_kebab_case_case;

struct StructAttributes {
    name: Option<String>,
    version: Option<String>,
    author: Option<String>,
    about: Option<String>,
}

pub fn derive_parser_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;

    // Parse struct attributes
    let struct_attrs = parse_struct_attributes(&input.attrs)?;
    let app_name = match struct_attrs.name {
        Some(name) => {
            let name = to_kebab_case_case(name.to_string());

            quote! { Some(#name) }
        },
        None => quote! { None },
    };

    let version = match struct_attrs.version {
        Some(version) => quote! { Some(#version) },
        None => quote! { None }
    };

    match input.data {
        Data::Struct(data_struct) => {
            match data_struct.fields {
                Fields::Named(fields) => {
                    let field_parsers = generate_field_parsers(&fields)?;
                    let field_assignments = generate_field_assignments(&fields)?;
                    let arg_definitions = generate_arg_definitions(&fields)?;
                    let global_arg_definitions = generate_global_arg_definitions(&fields)?;
                    let subcommand_definitions = generate_subcommand_definitions(&fields, &global_arg_definitions)?;

                    let expanded = quote! {
                        impl ::no_std_clap_core::parser::Parser for #name {
                            fn parse_args(args: &[::alloc::string::String]) -> ::core::result::Result<Self, ::no_std_clap_core::error::ParseError> {
                                use ::no_std_clap_core::command::Command;
                                use ::no_std_clap_core::subcommand::SubcommandInfo;
                                use ::no_std_clap_core::arg::arg_info::ArgInfo;
                                use ::no_std_clap_core::arg::from_arg::FromArg;
                                use ::no_std_clap_core::parser::{Subcommand, Args};
                                use ::alloc::string::ToString;

                                let mut cmd = Command::new(#app_name, #version);

                                #(cmd = cmd.arg(#arg_definitions);)*
                                cmd = cmd.arg(
                                    ArgInfo::new("help")
                                        .short('h')
                                        .long("help")
                                        .help("Prints help information")
                                        .global()
                                );

                                #(#subcommand_definitions)*

                                let parsed = cmd.parse(args)?;

                                #(#field_parsers)*

                                Ok(Self {
                                    #(#field_assignments)*
                                })
                            }
                        }
                    };

                    Ok(TokenStream::from(expanded))
                }
                _ => Err(Error::new_spanned(
                    name,
                    "Parser can only be derived for structs with named fields",
                )),
            }
        }
        _ => Err(Error::new_spanned(name, "Parser can only be derived for structs")),
    }
}

fn parse_struct_attributes(attrs: &[Attribute]) -> Result<StructAttributes, Error> {
    let mut struct_attrs = StructAttributes {
        name: None,
        version: None,
        author: None,
        about: None,
    };

    for attr in attrs {
        if attr.path().is_ident("clap") {
            match &attr.meta {
                Meta::List(_meta_list) => {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("name") {
                            let value: LitStr = meta.value()?.parse()?;
                            struct_attrs.name = Some(value.value());
                        }
                        else if meta.path.is_ident("version") {
                            let value: LitStr = meta.value()?.parse()?;
                            struct_attrs.version = Some(value.value());
                        }
                        else if meta.path.is_ident("author") {
                            let value: LitStr = meta.value()?.parse()?;
                            struct_attrs.author = Some(value.value());
                        }
                        else if meta.path.is_ident("about") {
                            let value: LitStr = meta.value()?.parse()?;
                            struct_attrs.about = Some(value.value());
                        }
                        Ok(())
                    })?;
                }
                _ => {}
            }
        }
        else if attr.path().is_ident("doc") {
            // Gather doc comments into about (if not explicitly set)
            if struct_attrs.about.is_none() {
                if let Meta::NameValue(meta_name_value) = &attr.meta {
                    if let Expr::Lit(expr_lit) = &meta_name_value.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            // Accumulate multiple lines into one string
                            let line = lit_str.value();
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                match &mut struct_attrs.about {
                                    Some(existing) => {
                                        existing.push(' ');
                                        existing.push_str(trimmed);
                                    }
                                    None => {
                                        struct_attrs.about = Some(trimmed.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(struct_attrs)
}