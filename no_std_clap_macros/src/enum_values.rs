use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Error, LitStr, Meta};
use crate::utils::to_kebab_case_case;

pub fn derive_enum_values_arg_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;

    let Data::Enum(DataEnum { variants, .. }) = &input.data else {
        return Err(Error::new_spanned(name, "#[derive(EnumValuesArg)] can only be used on enums"));
    };

    // Build match arms
    let mut arms = Vec::new();
    let mut variant_names = Vec::new();

    for variant in variants {
        let v_ident = &variant.ident;
        let mut variant_name = to_kebab_case_case(v_ident.to_string());

        // look for #[arg(name = "...")]
        for attr in &variant.attrs {
            if attr.path().is_ident("arg") {
                match &attr.meta {
                    Meta::List(_meta_list) => {
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("name") {
                                let value: LitStr = meta.value()?.parse()?;
                                variant_name = value.value().to_string();
                            }
                            Ok(())
                        })?;
                    },
                    _ => {}
                }
            }
        }

        variant_names.push(variant_name.clone());

        arms.push(quote! {
            #variant_name => Ok(Self::#v_ident),
        });
    }

    let variant_name_stringed = variant_names.join("|");

    let expanded = quote! {
        impl ::no_std_clap_core::arg::from_arg::FromArg for #name {
            fn from_arg(value: &str) -> Result<Self, ::no_std_clap_core::error::ParseError> {
                match value {
                    #(#arms)*
                    other => Err(::no_std_clap_core::error::ParseError::UnknownEnumVariant(other.to_string(), ::alloc::string::String::from(#variant_name_stringed))),
                }
            }
        }
    };

    Ok(TokenStream::from(expanded))
}