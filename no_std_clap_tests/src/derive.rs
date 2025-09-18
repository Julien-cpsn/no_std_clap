use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use no_std_clap_core::parser::Parser;
use no_std_clap_macros::Parser;

#[derive(Parser, Debug, PartialEq)]
#[clap(name = "myapp", version = "1.0")]
struct Args {
    #[arg(short, long, help = "Name to use")]
    name: String,

    #[arg(short, long, required)]
    count: i32,

    #[arg(short, long, global)]
    verbose: bool,

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
        "--count".to_string(),
        "42".to_string(),
        "--verbose".to_string(),
        "--list".to_string(),
        "5".to_string(),
    ];

    let result_1 = Args::parse_args(&args).unwrap();
    assert_eq!(result_1.name, "test");
    assert_eq!(result_1.count, 42);
    assert!(result_1.verbose);
    assert_eq!(result_1.list, vec![5]);
    assert_eq!(result_1.optional, None);
    assert_eq!(result_1.optional_with_default, Some(3));

    let result_2 = Args::parse_str("--name test --count 42 --verbose true --list 5").unwrap();
    assert_eq!(result_1, result_2);
}