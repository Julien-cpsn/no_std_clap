use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub struct ParsedArgs {
    args: BTreeMap<String, Vec<String>>,
    pub subcommand: Option<(String, Box<ParsedArgs>)>,
}

impl ParsedArgs {
    pub fn new() -> Self {
        Self {
            args: BTreeMap::new(),
            subcommand: None,
        }
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.args.entry(key).or_insert_with(Vec::new).push(value);
    }

    pub fn insert_flag(&mut self, key: String) {
        self.args.entry(key).or_insert_with(Vec::new);
    }

    pub fn set_subcommand(&mut self, name: String, args: ParsedArgs) {
        self.subcommand = Some((name, Box::new(args)));
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

    // Get subcommand name and args
    pub fn get_subcommand(&self) -> Option<(&str, &ParsedArgs)> {
        self.subcommand.as_ref().map(|(name, args)| (name.as_str(), args.as_ref()))
    }
}