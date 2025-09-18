use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Error, Fields, FieldsNamed, LitStr, Meta, Variant};
use crate::args::{generate_arg_info_for_args, generate_args_field_assignments, generate_args_field_parsers};
use crate::field::parse_field_attributes;
use crate::utils::{get_inner_type, to_kebab_case_case};

#[derive(Default)]
struct SubcommandVariantAttributes {
    name: Option<String>,
    about: Option<String>,
}

pub fn derive_subcommand_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;

    match input.data {
        Data::Enum(data_enum) => {
            let match_arms = generate_subcommand_match_arms(&data_enum)?;
            let subcommand_info_arms = generate_subcommand_info_arms(&data_enum)?;

            let expanded = quote! {
                impl ::no_std_clap_core::parser::Subcommand for #name {
                    fn from_subcommand(
                        name: &str,
                        args: &::no_std_clap_core::arg::parsed_arg::ParsedArgs,
                    ) -> ::core::result::Result<Self, ::no_std_clap_core::error::ParseError> {
                        use ::no_std_clap_core::parser::Args;
                        use ::alloc::string::ToString;

                        match name {
                            #(#match_arms)*
                            _ => Err(::no_std_clap_core::error::ParseError::UnknownArgument(name.to_string())),
                        }
                    }

                    fn subcommand_info() -> ::alloc::vec::Vec<::no_std_clap_core::subcommand::SubcommandInfo> {
                        use ::no_std_clap_core::subcommand::SubcommandInfo;
                        use ::no_std_clap_core::parser::Args;

                        ::alloc::vec![
                            #(#subcommand_info_arms)*
                        ]
                    }
                }
            };

            Ok(TokenStream::from(expanded))
        }
        _ => Err(Error::new_spanned(
            name,
            "Subcommand can only be derived for enums",
        )),
    }
}

// Generate subcommand definitions for the main command
pub fn generate_subcommand_definitions(fields: &FieldsNamed, global_arg_defs: &[proc_macro2::TokenStream]) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut definitions = Vec::new();

    for field in &fields.named {
        let field_attrs = parse_field_attributes(field)?;

        if field_attrs.subcommand {
            let field_type = &field.ty;

            // Extract the inner type from Option<T> if it's optional
            let subcommand_type = get_inner_type(field_type).unwrap_or_else(|| field_type);

            let definition = quote! {
                {
                    let mut subcommand_infos = <#subcommand_type as Subcommand>::subcommand_info();
                    for mut info in subcommand_infos {
                        #(
                            info = info.arg(#global_arg_defs);
                        )*

                        cmd = cmd.subcommand(info);
                    }
                }
            };
            definitions.push(definition);
        }
    }

    Ok(definitions)
}

// Generate match arms for subcommand enum variants
fn generate_subcommand_match_arms(data_enum: &DataEnum) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut arms = Vec::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        let variant_attrs = parse_subcommand_variant_attributes(variant)?;

        let command_name = variant_attrs.name.unwrap_or_else(|| to_kebab_case_case(variant_name.to_string()));

        match &variant.fields {
            Fields::Unit => {
                // Simple enum variant without fields
                let arm = quote! {
                    #command_name => Ok(Self::#variant_name),
                };
                arms.push(arm);
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                // Tuple variant with single field (e.g., Add(AddArgs))
                let field_type = &fields.unnamed.first().unwrap().ty;
                let arm = quote! {
                    #command_name => Ok(Self::#variant_name(<#field_type as Args>::from_args(args)?)),
                };
                arms.push(arm);
            }
            Fields::Named(fields) => {
                // Struct variant with named fields
                let field_parsers = generate_args_field_parsers(fields)?;
                let field_assignments = generate_args_field_assignments(fields)?;

                let arm = quote! {
                    #command_name => {
                        #(#field_parsers)*
                        Ok(Self::#variant_name {
                            #(#field_assignments)*
                        })
                    },
                };
                arms.push(arm);
            }
            _ => {
                return Err(Error::new_spanned(
                    variant,
                    "Subcommand variants can only have unit, single unnamed field, or named fields"
                ));
            }
        }
    }

    Ok(arms)
}

// Generate subcommand info arms for each variant
fn generate_subcommand_info_arms(data_enum: &DataEnum) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut arms = Vec::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        let variant_attrs = parse_subcommand_variant_attributes(variant)?;

        let command_name = variant_attrs.name.unwrap_or_else(|| {
            variant_name.to_string().to_lowercase().replace('_', "-")
        });

        let about = variant_attrs.about.as_deref().unwrap_or("");

        match &variant.fields {
            Fields::Unit => {
                // Simple enum variant without fields
                let arm = quote! {
                    SubcommandInfo::new(#command_name).about(#about),
                };
                arms.push(arm);
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                // Tuple variant with single field
                let field_type = &fields.unnamed.first().unwrap().ty;
                let arm = quote! {
                    {
                        let mut info = SubcommandInfo::new(#command_name).about(#about);
                        let arg_infos = <#field_type as Args>::arg_info();
                        for arg_info in arg_infos {
                            info = info.arg(arg_info);
                        }
                        info
                    },
                };
                arms.push(arm);
            }
            Fields::Named(fields) => {
                // Struct variant with named fields
                let arg_info_generation = generate_arg_info_for_args(fields)?;

                let arm = quote! {
                    {
                        let mut info = SubcommandInfo::new(#command_name).about(#about);
                        let arg_infos = ::alloc::vec![#(#arg_info_generation),*];
                        for arg_info in arg_infos {
                            info = info.arg(arg_info);
                        }
                        info
                    },
                };
                arms.push(arm);
            }
            _ => {
                return Err(Error::new_spanned(
                    variant,
                    "Subcommand variants can only have unit, single unnamed field, or named fields"
                ));
            }
        }
    }

    Ok(arms)
}

fn parse_subcommand_variant_attributes(variant: &Variant) -> Result<SubcommandVariantAttributes, Error> {
    let mut variant_attrs = SubcommandVariantAttributes::default();

    for attr in &variant.attrs {
        if attr.path().is_ident("command") {
            match &attr.meta {
                Meta::List(_) => {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("name") {
                            let value: LitStr = meta.value()?.parse()?;
                            variant_attrs.name = Some(value.value());
                        }
                        else if meta.path.is_ident("about") {
                            let value: LitStr = meta.value()?.parse()?;
                            variant_attrs.about = Some(value.value());
                        }
                        Ok(())
                    })?;
                }
                _ => {}
            }
        }
    }

    Ok(variant_attrs)
}