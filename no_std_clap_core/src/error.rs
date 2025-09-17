use alloc::string::String;
use core::fmt;
use core::fmt::Display;

// Error types
#[derive(Debug)]
pub enum ParseError {
    MissingArgument(String),
    InvalidValue(String),
    UnknownArgument(String),
    InvalidFormat(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingArgument(arg) => write!(f, "Missing required argument: {}", arg),
            ParseError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            ParseError::UnknownArgument(arg) => write!(f, "Unknown argument: {}", arg),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}