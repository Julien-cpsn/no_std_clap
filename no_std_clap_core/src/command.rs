use crate::arg::arg_info::ArgInfo;
use crate::error::ParseError;
use crate::subcommand::SubcommandInfo;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::arg::parsed_arg::ParsedArgs;

// Command structure for building parsers manually
pub struct Command {
    name: String,
    args: Vec<ArgInfo>,
    subcommands: Vec<SubcommandInfo>,
}

impl Command {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            args: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn arg(mut self, arg: ArgInfo) -> Self {
        self.args.push(arg);
        self
    }

    pub fn subcommand(mut self, subcommand: SubcommandInfo) -> Self {
        self.subcommands.push(subcommand);
        self
    }

    pub fn parse(&self, args: &[String]) -> Result<ParsedArgs, ParseError> {
        self.parse_with_subcommands(args, &self.args, &self.subcommands)
    }

    fn parse_with_subcommands(&self, args: &[String], current_args: &[ArgInfo], current_subcommands: &[SubcommandInfo]) -> Result<ParsedArgs, ParseError> {
        let mut result = ParsedArgs::new();
        let mut i = 0;

        while i < args.len() {
            let arg = &args[i];

            // Check if this is a subcommand
            if !arg.starts_with('-') {
                if let Some(subcommand_info) = current_subcommands.iter().find(|sc| sc.name == *arg) {
                    // Parse the remaining arguments as subcommand arguments
                    let remaining_args = &args[i + 1..];
                    let subcommand_result = self.parse_with_subcommands(
                        remaining_args,
                        &subcommand_info.args,
                        &subcommand_info.subcommands,
                    )?;
                    result.set_subcommand(arg.clone(), subcommand_result);
                    break; // Stop parsing after subcommand
                }
                else {
                    // Skip unknown positional arguments for now
                    i += 1;
                    continue;
                }
            }

            if arg.starts_with("--") {
                // Long argument
                let arg_name = &arg[2..];
                if let Some(arg_info) = current_args.iter().find(|a| a.long.as_ref().map_or(false, |l| l == arg_name)) {
                    if is_bool_flag(args, i) {
                        // Boolean flag - just mark as present
                        result.insert_flag(arg_info.name.clone());
                    }
                    else {
                        // Value argument
                        i += 1;
                        if i >= args.len() {
                            return Err(ParseError::MissingArgument(arg_name.to_string()));
                        }
                        result.insert(arg_info.name.clone(), args[i].clone());
                    }
                }
                else {
                    return Err(ParseError::UnknownArgument(arg_name.to_string()));
                }
            }
            else if arg.starts_with('-') && arg.len() == 2 {
                // Short argument
                let short_char = arg.chars().nth(1).unwrap();
                if let Some(arg_info) = current_args.iter().find(|a| a.short == Some(short_char)) {
                    if is_bool_flag(args, i) {
                        // Boolean flag - just mark as present
                        result.insert_flag(arg_info.name.clone());
                    }
                    else {
                        // Value argument
                        i += 1;
                        if i >= args.len() {
                            return Err(ParseError::MissingArgument(short_char.to_string()));
                        }
                        result.insert(arg_info.name.clone(), args[i].clone());
                    }
                }
                else {
                    return Err(ParseError::UnknownArgument(short_char.to_string()));
                }
            }
            i += 1;
        }

        // Check required arguments
        for arg_info in current_args {
            if arg_info.required && !result.contains_key(&arg_info.name) {
                return Err(ParseError::MissingArgument(arg_info.name.clone()));
            }
        }

        Ok(result)
    }
}

// Helper function to determine if an argument is a boolean flag
fn is_bool_flag(args: &[String], current_index: usize) -> bool {
    // If the next argument starts with '-' or we're at the end, treat as boolean
    let next_index = current_index + 1;
    if next_index >= args.len() {
        return true;
    }

    let next_arg = &args[next_index];
    if next_arg.starts_with('-') {
        return true;
    }

    // For explicit boolean values
    matches!(next_arg.to_lowercase().as_str(), "true" | "false" | "1" | "0" | "yes" | "no" | "on" | "off")
}