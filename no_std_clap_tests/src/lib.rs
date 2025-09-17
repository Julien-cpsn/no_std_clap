#![no_std]
extern crate alloc;

// Example usage and test
#[cfg(test)]
mod tests {
    use alloc::string::{String, ToString};
    use alloc::vec;
    use no_std_clap_core::arg::{ArgInfo, FromArg};
    use no_std_clap_core::command::Command;
    use no_std_clap_core::error::ParseError;
    use no_std_clap_core::parser::Parser;
    use no_std_clap_macros::Parser;

    #[derive(Debug, PartialEq)]
    struct TestArgs {
        name: String,
        count: i32,
        verbose: bool,
        optional: Option<String>,
    }

    // Manual implementation of what the derive macro would generate
    impl Parser for TestArgs {
        fn parse_args(args: &[String]) -> Result<Self, ParseError> {
            let cmd = Command::new("test")
                .arg(ArgInfo::new("name").long("name").short('n').required())
                .arg(ArgInfo::new("count").long("count").short('c').required())
                .arg(ArgInfo::new("verbose").long("verbose").short('v'))
                .arg(ArgInfo::new("optional").long("optional").short('o'));

            let parsed = cmd.parse(args)?;

            let name = parsed
                .get("name")
                .ok_or_else(|| ParseError::MissingArgument("name".to_string()))?;

            let count_str = parsed
                .get("count")
                .ok_or_else(|| ParseError::MissingArgument("count".to_string()))?;

            let verbose = parsed
                .get("verbose")
                .map(|_| true).unwrap_or(false);

            let optional = parsed
                .get("optional")
                .map(|s| s.clone());

            Ok(TestArgs {
                name: String::from_arg(name)?,
                count: i32::from_arg(count_str)?,
                verbose,
                optional
            })
        }
    }

    #[test]
    fn test_basic_parsing() {
        let args = vec![
            "--name".to_string(),
            "test".to_string(),
            "--count".to_string(),
            "42".to_string(),
            "--verbose".to_string(),
            "true".to_string(),
        ];

        let result_1 = TestArgs::parse_args(&args).unwrap();
        assert_eq!(result_1.name, "test");
        assert_eq!(result_1.count, 42);
        assert!(result_1.verbose);
        assert_eq!(result_1.optional, None);

        let result_2 = TestArgs::parse_str("--name test --count 42 --verbose true").unwrap();
        assert_eq!(result_1, result_2);
    }

    #[derive(Parser, Debug, PartialEq)]
    #[clap(name = "myapp", version = "1.0")]
    struct Args {
        #[arg(short, long, help = "Name to use")]
        name: String,

        #[arg(short, long, required)]
        count: i32,

        #[arg(short, long)]
        verbose: bool,

        // or default_value = "Default"
        #[arg(long)]
        optional: Option<String>,

        #[arg(skip)]
        computed: String, // Will use Default::default()
    }

    #[test]
    fn test_derive_parsing() {
        let args = vec![
            "--name".to_string(),
            "test".to_string(),
            "--count".to_string(),
            "42".to_string(),
            "--verbose".to_string(),
            "true".to_string(),
        ];

        let result_1 = Args::parse_args(&args).unwrap();
        assert_eq!(result_1.name, "test");
        assert_eq!(result_1.count, 42);
        assert!(result_1.verbose);
        assert_eq!(result_1.optional, None);

        let result_2 = Args::parse_str("--name test --count 42 --verbose true").unwrap();
        assert_eq!(result_1, result_2);
    }
}