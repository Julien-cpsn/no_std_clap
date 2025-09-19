use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use no_std_clap_core::arg::arg_info::ArgInfo;
use no_std_clap_core::arg::from_arg::FromArg;
use no_std_clap_core::command::Command;
use no_std_clap_core::error::ParseError;
use no_std_clap_core::parser::Parser;

#[derive(Debug, PartialEq)]
struct TestArgs {
    name: String,
    count: i32,
    verbose: bool,
    list: Vec<u8>,
    optional: Option<String>,
}

// Manual implementation of what the derive macro would generate
impl Parser for TestArgs {
    fn parse_args(args: &[String]) -> Result<Self, ParseError> {
        let cmd = Command::new(Some("test"), Some("Julien-cpsn"), Some("0.1.0"), None)
            .arg(ArgInfo::new("name").long("name").short('n').required())
            .arg(ArgInfo::new("count").long("count").short('c').required())
            .arg(ArgInfo::new("verbose").long("verbose").short('v').global())
            .arg(ArgInfo::new("list").long("list").short('l').multiple())
            .arg(ArgInfo::new("optional").long("optional").short('o'));

        let parsed = cmd.parse(args)?;

        let name = parsed
            .get("name")
            .ok_or_else(|| ParseError::MissingArgument("name".to_string()))?;

        let count_str = parsed
            .get("count")
            .ok_or_else(|| ParseError::MissingArgument("count".to_string()))?;

        let verbose = parsed.contains_key("verbose");

        let list = parsed
            .get_all("list")
            .into_iter()
            .map(|s| u8::from_arg(s).unwrap())
            .collect();

        let optional = parsed
            .get("optional")
            .map(|s| s.clone());

        Ok(TestArgs {
            name: String::from_arg(name)?,
            count: i32::from_arg(count_str)?,
            verbose,
            list,
            optional
        })
    }

    fn get_help() -> String {
        let cmd = Command::new(Some("test"), Some("Julien-cpsn"), Some("0.1.0"), None)
            .arg(ArgInfo::new("name").long("name").short('n').required())
            .arg(ArgInfo::new("count").long("count").short('c').required())
            .arg(ArgInfo::new("verbose").long("verbose").short('v').global())
            .arg(ArgInfo::new("list").long("list").short('l').multiple())
            .arg(ArgInfo::new("optional").long("optional").short('o'));

        cmd.get_help()
    }
}

#[test]
fn test_basic_parsing() {
    let args = vec![
        "--name".to_string(),
        "test".to_string(),
        "--count".to_string(),
        "42".to_string(),
        "--list".to_string(),
        "5".to_string(),
        "--verbose".to_string(),
    ];

    let result_1 = TestArgs::parse_args(&args).unwrap();
    assert_eq!(result_1.name, "test");
    assert_eq!(result_1.count, 42);
    assert!(result_1.verbose);
    assert_eq!(result_1.list, vec![5]);
    assert_eq!(result_1.optional, None);

    let result_2 = TestArgs::parse_str("--name test --count 42 --list 5 --verbose").unwrap();
    assert_eq!(result_1, result_2);
}