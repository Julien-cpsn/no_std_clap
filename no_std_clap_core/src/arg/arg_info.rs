use alloc::string::{String, ToString};

// Argument metadata
#[derive(Debug, Clone)]
pub struct ArgInfo {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub help: Option<String>,
    pub required: bool,
    pub multiple: bool,
    pub global: bool,
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
            global: false,
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

    pub fn global(mut self) -> Self {
        self.global = true;
        self
    }
}
