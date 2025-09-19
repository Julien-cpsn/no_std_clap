use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::arg::arg_info::ArgInfo;
use crate::arg::parsed_arg::ParsedArgs;
use crate::error::ParseError;
use crate::help::get_help;
use crate::subcommand::SubcommandInfo;

// Main parser trait
pub trait Parser: Sized {
    fn parse_args(args: &[String]) -> Result<Self, ParseError>;

    fn parse_from<I, T>(args: I) -> Result<Self, ParseError>
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let args: Vec<String> = args.into_iter().map(|s| s.into()).collect();

        if args.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        Self::parse_args(&args)
    }

    fn parse_str(input: &str) -> Result<Self, ParseError> {
        let input = input.trim();

        if input.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let args = parse_command_line(input)?;
        Self::parse_args(&args)
    }

    fn get_help() -> String;
}

// Trait for types that can be used as subcommands
pub trait Subcommand: Sized {
    fn from_subcommand(name: &str, parents_name: Option<String>, args: &ParsedArgs) -> Result<Self, ParseError>;
    fn subcommand_info() -> Vec<SubcommandInfo>;
    fn get_help() -> String {
        let mut out = String::new();
        let info = Self::subcommand_info();

        get_help(&mut out, None, &Vec::new(), &Vec::new(), &info);

        out
    }
}

// Trait for arguments that can have subcommands
pub trait Args: Sized {
    fn from_args(args: &ParsedArgs) -> Result<Self, ParseError>;
    fn arg_info() -> Vec<ArgInfo>;
    fn get_help(name: String, parents_name: Option<String>, help: Option<String>) -> String;
}

// Command line string parsing function
fn parse_command_line(input: &str) -> Result<Vec<String>, ParseError> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;
    let mut quote_char = '"';

    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if escape_next {
            current_arg.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_quotes => {
                escape_next = true;
            }
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
            }
            ch if ch == quote_char && in_quotes => {
                in_quotes = false;
            }
            ' ' | '\t' if !in_quotes => {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
                // Skip multiple whitespace
                while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
                    chars.next();
                }
            }
            ch => {
                current_arg.push(ch);
            }
        }
    }

    if in_quotes {
        return Err(ParseError::InvalidFormat("Unclosed quote in command line".to_string()));
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    Ok(args)
}


// Additional utility functions
pub fn parse_env() -> Vec<String> {
    // In a real no_std environment, you'd need to provide args differently
    // This is just a placeholder for the interface
    Vec::new()
}