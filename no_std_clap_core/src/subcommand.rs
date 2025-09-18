use crate::arg::arg_info::ArgInfo;
use crate::arg::parsed_arg::ParsedArgs;
use crate::error::ParseError;
use crate::help::get_help;
use crate::parser::Subcommand;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Write;

// Subcommand information
#[derive(Debug, Clone)]
pub struct SubcommandInfo {
    pub name: String,
    pub about: Option<String>,
    pub args: Vec<ArgInfo>,
    pub subcommands: Vec<SubcommandInfo>,
}

impl SubcommandInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            about: None,
            args: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    pub fn about(mut self, about: &str) -> Self {
        self.about = Some(about.to_string());
        self
    }

    pub fn arg(mut self, arg: ArgInfo) -> Self {
        self.args.push(arg);
        self
    }

    pub fn subcommand(mut self, subcommand: SubcommandInfo) -> Self {
        self.subcommands.push(subcommand);
        self
    }

    pub fn get_help(&self) -> String {
        let mut out = String::new();
        
        if let Some(about) = &self.about {
            write!(out, "{}", about).unwrap();
        }

        writeln!(out).unwrap();
        writeln!(out).unwrap();

        get_help(&mut out, None, &self.args, &self.subcommands);

        out
    }
}

// Implement Subcommand for Option<T> where T: Subcommand
impl<T: Subcommand> Subcommand for Option<T> {
    fn from_subcommand(name: &str, args: &ParsedArgs) -> Result<Self, ParseError> {
        T::from_subcommand(name, args).map(Some)
    }

    fn subcommand_info() -> Vec<SubcommandInfo> {
        T::subcommand_info()
    }
}