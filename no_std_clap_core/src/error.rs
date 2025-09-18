use alloc::string::String;
use core::fmt;
use core::fmt::Display;

// Error types
#[derive(Debug)]
pub enum ParseError {
    EmptyInput,
    Help(String),
    MissingArgument(String),
    InvalidValue(String),
    UnknownArgument(String),
    UnknownSubcommand,
    InvalidFormat(String),
    UnknownEnumVariant(String, String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "Empty input"),
            ParseError::Help(help) => write!(f, "{}", help),
            ParseError::MissingArgument(arg) => write!(f, "Missing required argument: {}", arg),
            ParseError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            ParseError::UnknownArgument(arg) => write!(f, "Unknown argument: {}", arg),
            ParseError::UnknownSubcommand => write!(f, "Unknown command"),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::UnknownEnumVariant(value, possible_values) => write!(f, "Invalid value: {}, possible values are: {}", value, possible_values),
        }
    }
}