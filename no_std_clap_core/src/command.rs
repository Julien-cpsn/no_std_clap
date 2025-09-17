use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::arg::ArgInfo;
use crate::error::ParseError;

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

    pub fn parse(&self, args: &[String]) -> Result<BTreeMap<String, String>, ParseError> {
        let mut result = BTreeMap::new();
        let mut i = 0;

        while i < args.len() {
            let arg = &args[i];

            if arg.starts_with("--") {
                // Long argument
                let arg_name = &arg[2..];
                if let Some(arg_info) = self.args.iter().find(|a| a.long.as_ref().map_or(false, |l| l == arg_name)) {
                    i += 1;
                    if i >= args.len() {
                        return Err(ParseError::MissingArgument(arg_name.to_string()));
                    }
                    result.insert(arg_info.name.clone(), args[i].clone());
                }
                else {
                    return Err(ParseError::UnknownArgument(arg_name.to_string()));
                }
            }
            else if arg.starts_with('-') && arg.len() == 2 {
                // Short argument
                let short_char = arg.chars().nth(1).unwrap();
                if let Some(arg_info) = self.args.iter().find(|a| a.short == Some(short_char)) {
                    i += 1;
                    if i >= args.len() {
                        return Err(ParseError::MissingArgument(short_char.to_string()));
                    }
                    result.insert(arg_info.name.clone(), args[i].clone());
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
