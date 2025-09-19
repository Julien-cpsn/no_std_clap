use crate::arg::arg_info::ArgInfo;
use crate::arg::parsed_arg::ParsedArgs;
use crate::error::ParseError;
use crate::help::get_help;
use crate::subcommand::SubcommandInfo;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Write;

// Command structure for building parsers manually
#[derive(Debug, Clone)]
pub struct Command {
    name: Option<String>,
    version: Option<String>,
    args: Vec<ArgInfo>,
    global_args: Vec<ArgInfo>,
    subcommands: Vec<SubcommandInfo>,
}

impl Command {
    pub fn new(name: Option<&str>, version: Option<&str>) -> Self {
        Self {
            name: name.map(|v| v.to_string()),
            version: version.map(|v| v.to_string()),
            args: Vec::new(),
            global_args: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_ref().map(|v| v.as_str())
    }

    pub fn get_version(&self) -> Option<&str> {
        self.version.as_ref().map(|v| v.as_str())
    }

    pub fn arg(mut self, arg: ArgInfo) -> Self {
        if arg.global {
            self.global_args.push(arg);
        }
        else {
            self.args.push(arg);
        }
        self
    }

    pub fn subcommand(mut self, subcommand: SubcommandInfo) -> Self {
        self.subcommands.push(subcommand);
        self
    }

    pub fn parse(&self, args: &[String]) -> Result<ParsedArgs, ParseError> {
        self.parse_with_subcommands(args, &self.args, &self.global_args, &self.subcommands)
    }

    fn parse_with_subcommands(&self, args: &[String], current_args: &[ArgInfo], global_args: &[ArgInfo], current_subcommands: &[SubcommandInfo]) -> Result<ParsedArgs, ParseError> {
        let mut result = ParsedArgs::new();
        let mut i = 0;

        while i < args.len() {
            let arg = &args[i];

            // Check if this is a subcommand
            if !arg.starts_with('-') {
                if let Some(subcommand_info) = current_subcommands.iter().find(|sc| sc.name == *arg) {
                    // Parse the remaining arguments as subcommand arguments
                    let remaining_args = &args[i + 1..];

                    let subcommand_result = match self.parse_with_subcommands(remaining_args, &subcommand_info.args, global_args, &subcommand_info.subcommands) {
                        Ok(subcommand_result) => subcommand_result,
                        Err(e) => return Err(e),
                    };

                    result.set_subcommand(arg.clone(), subcommand_result);

                    // Stop parsing after subcommand
                    break;
                }
                else if let Some(arg_info) = current_args.iter().find(|a| a.short.is_none() && a.long.is_none()) {
                    // Positional argument
                    result.insert(arg_info.name.clone(), arg.clone());
                }
                else {
                    // Unknown positional argument -> error or ignore
                    return Err(ParseError::UnknownArgument(arg.clone()));
                }
            }

            // Determine argument metadata (current + global)
            let all_args: Vec<&ArgInfo> = current_args.iter().chain(global_args.iter()).collect();

            if arg.starts_with("--") {
                // Long argument
                let arg_name = &arg[2..];
                if let Some(arg_info) = all_args.iter().find(|a| a.long.as_ref().map_or(false, |l| l == arg_name)) {
                    if arg_info.count {
                        // increment once for each occurrence
                        result.increment(arg_info.name.clone());
                    }
                    else if is_bool_flag(args, i) {
                        // Boolean flag - just mark as present
                        result.insert_flag(arg_info.name.clone());
                    }
                    else {
                        // Value argument
                        i += 1;
                        result.insert(arg_info.name.clone(), args[i].clone());
                    }
                }
                else {
                    return Err(ParseError::UnknownArgument(arg_name.to_string()));
                }
            }
            else if arg.starts_with('-') && arg.len() >= 2 {
                // Handle clusters: e.g. -vvv or -abc
                for short_char in arg.chars().skip(1) {
                    if let Some(arg_info) = all_args.iter().find(|a| a.short == Some(short_char)) {
                        if arg_info.count {
                            // increment once for each occurrence
                            result.increment(arg_info.name.clone());
                        }
                        else if is_bool_flag(args, i) {
                            result.insert_flag(arg_info.name.clone());
                        }
                        else {
                            let rest: String = arg.chars().skip_while(|c| *c != short_char).skip(1).collect();
                            if !rest.is_empty() {
                                result.insert(arg_info.name.clone(), rest);
                            }
                            else {
                                i += 1;
                                result.insert(arg_info.name.clone(), args[i].clone());
                            }
                            // stop further processing of this cluster
                            break;
                        }
                    }
                    else {
                        return Err(ParseError::UnknownArgument(short_char.to_string()));
                    }
                }
            }

            i += 1;
        }

        Ok(result)
    }

    pub fn get_help(&self) -> String {
        let mut out = String::new();

        if let Some(name) = self.get_name() {
            write!(out, "{}", name).unwrap();
        }

        if let Some(version) = &self.version {
            write!(out, " {}", version).unwrap();
        }

        get_help(&mut out, self.name.as_ref(), &self.args, &self.global_args, &self.subcommands);

        out
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