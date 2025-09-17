use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::arg::arg_info::ArgInfo;
use crate::arg::parsed_arg::ParsedArgs;
use crate::error::ParseError;
use crate::parser::Subcommand;

// Subcommand information
#[derive(Clone)]
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