use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::error::ParseError;

// Trait for types that can be parsed from command line arguments
pub trait FromArg: Sized {
    fn from_arg(arg: &str) -> Result<Self, ParseError>;
}

// Implement FromArg for primitive types
impl FromArg for String {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        Ok(arg.to_string())
    }
}

impl FromArg for i8 {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.parse().map_err(|_| ParseError::InvalidValue(format!("Cannot parse '{}' as i8", arg)))
    }
}

impl FromArg for i16 {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.parse().map_err(|_| ParseError::InvalidValue(format!("Cannot parse '{}' as i16", arg)))
    }
}

impl FromArg for i32 {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.parse().map_err(|_| ParseError::InvalidValue(format!("Cannot parse '{}' as i32", arg)))
    }
}

impl FromArg for i64 {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.parse().map_err(|_| ParseError::InvalidValue(format!("Cannot parse '{}' as i64", arg)))
    }
}

impl FromArg for u8 {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.parse().map_err(|_| ParseError::InvalidValue(format!("Cannot parse '{}' as u8", arg)))
    }
}

impl FromArg for u16 {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.parse().map_err(|_| ParseError::InvalidValue(format!("Cannot parse '{}' as u16", arg)))
    }
}

impl FromArg for u32 {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.parse().map_err(|_| ParseError::InvalidValue(format!("Cannot parse '{}' as u32", arg)))
    }
}

impl FromArg for u64 {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.parse().map_err(|_| ParseError::InvalidValue(format!("Cannot parse '{}' as u64", arg)))
    }
}

impl FromArg for f32 {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.parse().map_err(|_| ParseError::InvalidValue(format!("Cannot parse '{}' as f32", arg)))
    }
}

impl FromArg for f64 {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.parse().map_err(|_| ParseError::InvalidValue(format!("Cannot parse '{}' as f64", arg)))
    }
}

impl FromArg for bool {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        match arg.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(ParseError::InvalidValue(format!("Cannot parse '{}' as bool", arg))),
        }
    }
}

// Optional types
impl<T: FromArg> FromArg for Option<T> {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        Ok(Some(T::from_arg(arg)?))
    }
}

// Vec types for multiple values
impl<T: FromArg> FromArg for Vec<T> {
    fn from_arg(arg: &str) -> Result<Self, ParseError> {
        arg.split(',')
            .map(|s| T::from_arg(s.trim()))
            .collect()
    }
}

// Argument metadata
#[derive(Debug, Clone)]
pub struct ArgInfo {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub help: Option<String>,
    pub required: bool,
    pub multiple: bool,
}

impl ArgInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            short: None,
            long: None,
            help: None,
            required: false,
            multiple: false,
        }
    }

    pub fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    pub fn long(mut self, long: &str) -> Self {
        self.long = Some(long.to_string());
        self
    }

    pub fn help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }
}
