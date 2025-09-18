use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Error, Fields, LitStr, Meta};
use crate::args::{generate_arg_definitions, generate_global_arg_definitions};
use crate::field::{generate_field_assignments, generate_field_parsers};
use crate::subcommand::generate_subcommand_definitions;
use crate::utils::to_kebab_case_case;

#[derive(Default)]
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
    let app_name = struct_attrs
        .name
        .unwrap_or_else(|| to_kebab_case_case(name.to_string()));

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

                                let mut cmd = Command::new(#app_name);
                                #(cmd = cmd.arg(#arg_definitions);)*
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
    let mut struct_attrs = StructAttributes::default();

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
    }

    Ok(struct_attrs)
}