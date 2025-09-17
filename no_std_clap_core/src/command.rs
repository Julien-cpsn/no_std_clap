use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::arg::ArgInfo;
use crate::error::ParseError;

pub struct ParsedArgs {
    args: BTreeMap<String, Vec<String>>,
}

impl ParsedArgs {
    fn new() -> Self {
        Self {
            args: BTreeMap::new(),
        }
    }

    fn insert(&mut self, key: String, value: String) {
        self.args.entry(key).or_insert_with(Vec::new).push(value);
    }

    fn insert_flag(&mut self, key: String) {
        self.args.entry(key).or_insert_with(Vec::new);
    }

    // Get the first value for an argument (for single-value arguments)
    pub fn get(&self, key: &str) -> Option<&String> {
        self.args.get(key)?.first()
    }

    // Get all values for an argument (for Vec arguments)
    pub fn get_all(&self, key: &str) -> Vec<&str> {
        self.args.get(key)
            .map(|values| values.iter().map(|s| s.as_str()).collect())
            .unwrap_or_else(Vec::new)
    }

    // Check if an argument was provided (for boolean flags)
    pub fn contains_key(&self, key: &str) -> bool {
        self.args.contains_key(key)
    }
}

// Command structure for building parsers manually
pub struct Command {
    name: String,
    args: Vec<ArgInfo>,
}

impl Command {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: ArgInfo) -> Self {
        self.args.push(arg);
        self
    }

    pub fn parse(&self, args: &[String]) -> Result<ParsedArgs, ParseError> {
        let mut result = ParsedArgs::new();
        let mut i = 0;

        while i < args.len() {
            let arg = &args[i];

            if arg.starts_with("--") {
                // Long argument
                let arg_name = &arg[2..];
                if let Some(arg_info) = self.args.iter().find(|a| a.long.as_ref().map_or(false, |l| l == arg_name)) {
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
                if let Some(arg_info) = self.args.iter().find(|a| a.short == Some(short_char)) {
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
            else {
                // Positional argument - for now, skip
                i += 1;
                continue;
            }
            i += 1;
        }

        // Check required arguments
        for arg_info in &self.args {
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