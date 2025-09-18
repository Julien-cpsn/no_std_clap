extern crate proc_macro;
mod utils;
mod parser;
mod subcommand;
mod args;
mod enum_values;
mod field;

use crate::args::derive_args_impl;
use crate::enum_values::derive_enum_values_arg_impl;
use crate::parser::derive_parser_impl;
use crate::subcommand::derive_subcommand_impl;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Parser, attributes(arg, clap, command))]
pub fn derive_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive_parser_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_derive(Subcommand, attributes(command))]
pub fn derive_subcommand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive_subcommand_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_derive(Args, attributes(arg, clap))]
pub fn derive_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive_args_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_derive(EnumValuesArg, attributes(arg))]
pub fn derive_enum_values_arg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive_enum_values_arg_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
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