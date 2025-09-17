extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Error, ExprLit, Field, Fields, FieldsNamed, GenericArgument, LitStr, Meta, PathArguments, Type};

#[proc_macro_derive(Parser, attributes(arg, clap))]
pub fn derive_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive_parser_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}

fn derive_parser_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;

    // Parse struct attributes
    let struct_attrs = parse_struct_attributes(&input.attrs)?;
    let app_name = struct_attrs.name.unwrap_or_else(|| name.to_string().to_lowercase());

    match input.data {
        Data::Struct(data_struct) => {
            match data_struct.fields {
                Fields::Named(fields) => {
                    let field_parsers = generate_field_parsers(&fields)?;
                    let field_assignments = generate_field_assignments(&fields)?;
                    let arg_definitions = generate_arg_definitions(&fields)?;

                    let expanded = quote! {
                        impl ::no_std_clap_core::parser::Parser for #name {
                            fn parse_args(args: &[::alloc::string::String]) -> ::core::result::Result<Self, ::no_std_clap_core::error::ParseError> {
                                use ::no_std_clap_core::command::Command;
                                use ::no_std_clap_core::arg::{ArgInfo, FromArg};
                                use ::alloc::string::ToString;

                                let cmd = Command::new(#app_name)
                                    #(#arg_definitions)*;

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
                    "Parser can only be derived for structs with named fields"
                )),
            }
        }
        _ => Err(Error::new_spanned(
            name,
            "Parser can only be derived for structs"
        )),
    }
}

#[derive(Default)]
struct StructAttributes {
    name: Option<String>,
    version: Option<String>,
    author: Option<String>,
    about: Option<String>,
}

#[derive(Default)]
struct FieldAttributes {
    short: Option<char>,
    long: Option<String>,
    help: Option<String>,
    required: bool,
    multiple: bool,
    default_value: Option<ExprLit>,
    skip: bool,
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

fn parse_field_attributes(field: &Field) -> Result<FieldAttributes, Error> {
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
                            let value: ExprLit = meta.value()?.parse()?;
                            field_attrs.default_value = Some(value);
                        }
                        else if meta.path.is_ident("skip") {
                            field_attrs.skip = true;
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
    }

    Ok(field_attrs)
}

fn generate_field_parsers(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut parsers = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_attrs = parse_field_attributes(field)?;

        if field_attrs.skip {
            continue;
        }

        let field_name_str = field_name.to_string();
        let var_name = format_ident!("parsed_{}", field_name);

        let is_optional = is_option_type(&field.ty);
        //let is_vec = is_vec_type(&field.ty);
        let is_bool = is_bool_type(&field.ty);

        let parser = if is_bool {
            // Boolean flags don't take values
            quote! {
                let #var_name = parsed.contains_key(#field_name_str);
            }
        }
        else if field_attrs.required && !is_optional {
            quote! {
                let #var_name = parsed.get(#field_name_str)
                    .ok_or_else(|| ::no_std_clap_core::error::ParseError::MissingArgument(#field_name_str.to_string()))?;
            }
        }
        else if let Some(default) = &field_attrs.default_value {
            quote! {
                let #var_name = parsed.get(#field_name_str)
                    .map(|s| Some(s.as_str()))
                    .unwrap_or(Some(#default));
            }
        }
        else {
            quote! {
                let #var_name = parsed.get(#field_name_str).map(|s| s.as_str());
            }
        };

        parsers.push(parser);
    }

    Ok(parsers)
}

fn generate_field_assignments(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>, Error> {
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
        //let is_vec = is_vec_type(field_type);
        let is_bool = is_bool_type(field_type);

        let mut assignment = if is_bool {
            quote! {
                #field_name: #var_name
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
        else {
            // Non-optional, non-required field without default - use Default
            quote! {
                #field_name: match #var_name {
                    Some(s) => <#field_type as FromArg>::from_arg(s)?,
                    None => ::core::default::Default::default(),
                }
            }
        };

        assignment.extend(quote! { , });

        assignments.push(assignment);
    }

    Ok(assignments)
}

fn generate_arg_definitions(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut definitions = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_attrs = parse_field_attributes(field)?;

        if field_attrs.skip {
            continue;
        }

        let field_name_str = field_name.to_string();
        let is_bool = is_bool_type(&field.ty);

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
        else if !is_bool {
            // Auto-generate long flag from field name
            let long_name = field_name_str.replace('_', "-");
            arg_info_def.extend(quote! {
                .long(&#long_name)
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

        if field_attrs.multiple {
            arg_info_def.extend(quote! {
                .multiple()
            });
        }

        let arg_def = quote! {
            .arg(#arg_info_def)
        };

        definitions.push(arg_def);
    }

    Ok(definitions)
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn is_vec_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Vec";
        }
    }
    false
}

fn is_bool_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "bool";
        }
    }
    false
}

fn get_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" || segment.ident == "Vec" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

// Additional attribute macros
#[proc_macro_attribute]
pub fn arg(_args: TokenStream, input: TokenStream) -> TokenStream {
    // This is a marker attribute - the actual processing happens in the derive macro
    input
}

#[proc_macro_attribute]
pub fn clap(_args: TokenStream, input: TokenStream) -> TokenStream {
    // This is a marker attribute - the actual processing happens in the derive macro
    input
}