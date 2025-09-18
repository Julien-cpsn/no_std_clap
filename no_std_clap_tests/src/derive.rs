use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use no_std_clap_core::error::ParseError;
use no_std_clap_core::parser::Parser;
use no_std_clap_macros::Parser;

#[derive(Parser, Debug, PartialEq)]
#[clap(name = "myapp", version = "1.0")]
struct Args {
    #[arg(short, long, help = "Name to use")]
    name: String,

    #[arg(short, long, required)]
    number: i32,

    #[arg(short, long, count, global)]
    verbose: usize,

    #[arg(short, long)]
    list: Vec<u8>,

    #[arg(long)]
    optional: Option<u16>,

    #[arg(long, default_value = "3")]
    optional_with_default: Option<u16>,

    #[arg(skip)]
    computed: String, // Will use Default::default()
}

#[test]
fn test_derive_parsing() {
    let args = vec![
        "--name".to_string(),
        "test".to_string(),
        "--number".to_string(),
        "42".to_string(),
        "--list".to_string(),
        "5".to_string(),
        "--list".to_string(),
        "6".to_string(),
        "--verbose".to_string(),
    ];

    let result_1 = Args::parse_args(&args).unwrap();
    assert_eq!(result_1.name, "test");
    assert_eq!(result_1.number, 42);
    assert_eq!(result_1.verbose, 1);
    assert_eq!(result_1.list, vec![5, 6]);
    assert_eq!(result_1.optional, None);
    assert_eq!(result_1.optional_with_default, Some(3));

    let result_2 = Args::parse_str("--name test --number 42 -v --list 5 --list 6").unwrap();
    assert_eq!(result_1, result_2);
}


#[test]
fn test_help_parsing() {
    let args = vec![
        "--name".to_string(),
        "test".to_string(),
        "--number".to_string(),
        "42".to_string(),
        "--list".to_string(),
        "5".to_string(),
        "--verbose".to_string(),
        "--help".to_string(),
    ];

    if let Err(ParseError::Help(help)) = Args::parse_args(&args) {
        assert!(!help.is_empty());
    }
}