use crate::args::{generate_arg_info_for_args, generate_args_field_assignments, generate_args_field_parsers};
use crate::field::parse_field_attributes;
use crate::utils::{get_inner_type, to_kebab_case_case};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Error, Expr, Fields, FieldsNamed, Lit, LitStr, Meta, Variant};

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
                    fn from_subcommand(name: &str, parents_name: Option<::alloc::string::String>, args: &::no_std_clap_core::arg::parsed_arg::ParsedArgs) -> ::core::result::Result<Self, ::no_std_clap_core::error::ParseError> {
                        use ::no_std_clap_core::parser::Args;

                        match name {
                            #(#match_arms)*
                            _ => Err(::no_std_clap_core::error::ParseError::UnknownArgument(::alloc::string::String::from(name))),
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
                    let mut subcommand_infos = <#subcommand_type as no_std_clap_core::parser::Subcommand>::subcommand_info();
                    for mut info in subcommand_infos {
                        #(
                            info = info.arg(#global_arg_defs);
                        )*

                        info = info.arg(
                            ArgInfo::new("help")
                                .short('h')
                                .long("help")
                                .help("Prints help information")
                                .global()
                        );

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

        let variant_is_subcommand = enum_variant_is_subcommand(variant);

        match &variant.fields {
            Fields::Unit => {
                arms.push(quote! {
                    #command_name => Ok(Self::#variant_name),
                });
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field = &fields.unnamed.first().unwrap();
                let field_type = &field.ty;
                let field_attrs = parse_field_attributes(field)?;

                if field_attrs.subcommand || variant_is_subcommand {
                    // Nested subcommand
                    arms.push(quote! {
                        #command_name => {
                            if let Some((sub_name, sub_args)) = args.subcommand.as_ref() {
                                let parents_name = match parents_name {
                                    Some(parents_name) => ::alloc::format!("{} {}", parents_name, name),
                                    None => ::alloc::string::String::from(name)
                                };

                                Ok(Self::#variant_name(
                                    <#field_type as ::no_std_clap_core::parser::Subcommand>::from_subcommand(sub_name, Some(parents_name), sub_args)?
                                ))
                            }
                            else {
                                let help = <Self as ::no_std_clap_core::parser::Subcommand>::subcommand_info()
                                    .into_iter()
                                    .find(|info| info.name == name)
                                    .unwrap()
                                    .get_help(parents_name);

                                Err(::no_std_clap_core::error::ParseError::Help(help))
                            }
                        },
                    });
                }
                else {
                    let about = match variant_attrs.about {
                        Some(about) => quote! { Some(::alloc::string::String::from(#about)) },
                        None => quote! { None }
                    };

                    // Plain Args struct
                    arms.push(quote! {
                        #command_name => if args.args.is_empty() || args.args.contains_key("help") {
                            let help = <#field_type as ::no_std_clap_core::parser::Args>::get_help(::alloc::string::String::from(name), parents_name, #about);
                            Err(::no_std_clap_core::error::ParseError::Help(help))
                        }
                        else {
                            Ok(Self::#variant_name(<#field_type as ::no_std_clap_core::parser::Args>::from_args(args)?))
                        },
                    });
                }
            }
            Fields::Unnamed(fields) => {
                let field = &fields.unnamed.first().unwrap();
                let field_type = &field.ty;

                let about = match variant_attrs.about {
                    Some(about) => quote! { Some(::alloc::string::String::from(#about)) },
                    None => quote! { None }
                };

                // Plain Args struct
                arms.push(quote! {
                    #command_name => if args.args.is_empty() || args.args.contains_key("help") {
                        let help = <#field_type as ::no_std_clap_core::parser::Args>::get_help(::alloc::string::String::from(name), parents_name, #about);
                        Err(::no_std_clap_core::error::ParseError::Help(help))
                    }
                    else {
                        Ok(Self::#variant_name(<#field_type as ::no_std_clap_core::parser::Args>::from_args(args)?))
                    },
                });
            },
            Fields::Named(fields) => {
                let field_parsers = generate_args_field_parsers(fields)?;
                let field_assignments = generate_args_field_assignments(fields)?;

                arms.push(quote! {
                    #command_name => {
                        #(#field_parsers)*
                        Ok(Self::#variant_name {
                            #(#field_assignments)*
                        })
                    },
                });
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
        let command_name = variant_attrs.name.unwrap_or_else(|| to_kebab_case_case(variant_name.to_string()));
        let about = variant_attrs.about.as_deref().unwrap_or("");

        let variant_is_subcommand = enum_variant_is_subcommand(variant);

        match &variant.fields {
            Fields::Unit => {
                arms.push(quote! {
                    SubcommandInfo::new(#command_name).about(#about),
                });
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field = &fields.unnamed.first().unwrap();
                let field_type = &field.ty;
                let field_attrs = parse_field_attributes(field)?;

                if field_attrs.subcommand || variant_is_subcommand {
                    arms.push(quote! {
                        {
                            let mut info = SubcommandInfo::new(#command_name).about(#about);
                            let subs = <#field_type as no_std_clap_core::parser::Subcommand>::subcommand_info();
                            for sub in subs {
                                info = info.subcommand(sub);
                            }
                            info
                        },
                    });
                }
                else {
                    arms.push(quote! {
                        {
                            let mut info = SubcommandInfo::new(#command_name).about(#about);
                            let arg_infos = <#field_type as Args>::arg_info();
                            for arg_info in arg_infos {
                                info = info.arg(arg_info);
                            }
                            info
                        },
                    });
                }
            }
            Fields::Named(fields) => {
                let arg_info_generation = generate_arg_info_for_args(fields)?;

                arms.push(quote! {
                    {
                        let mut info = SubcommandInfo::new(#command_name).about(#about);
                        let arg_infos = ::alloc::vec![#(#arg_info_generation),*];
                        for arg_info in arg_infos {
                            info = info.arg(arg_info);
                        }
                        info
                    },
                });
            }
            _ => {
                return Err(Error::new_spanned(
                    variant,
                    "Subcommand variants can only have unit, single unnamed field, or named fields",
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
        else if attr.path().is_ident("doc") {
            // Gather doc comments into about (if not explicitly set)
            if variant_attrs.about.is_none() {
                if let Meta::NameValue(meta_name_value) = &attr.meta {
                    if let Expr::Lit(expr_lit) = &meta_name_value.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            // Accumulate multiple lines into one string
                            let line = lit_str.value();
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                match &mut variant_attrs.about {
                                    Some(existing) => {
                                        existing.push(' ');
                                        existing.push_str(trimmed);
                                    }
                                    None => {
                                        variant_attrs.about = Some(trimmed.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(variant_attrs)
}

fn enum_variant_is_subcommand(variant: &Variant) -> bool{
    variant.attrs.iter().any(|attr| {
        let mut is_subcommand = false;
        if attr.path().is_ident("command") {
            match &attr.meta {
                Meta::List(_) => {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("subcommand") {
                            is_subcommand = true;
                        }

                        Ok(())
                    }).ok();
                }
                Meta::Path(_) => {
                    is_subcommand = true;
                },
                _ => {}
            }
        }

        is_subcommand
    })
}