use alloc::string::ToString;
use alloc::vec;
use no_std_clap_core::parser::Parser;
use no_std_clap_macros::{EnumValuesArg, Parser};

#[derive(Parser, Debug, PartialEq)]
#[clap(name = "myapp", version = "1.0")]
struct Args {
    #[arg(short, long)]
    name: Names,

}

#[derive(EnumValuesArg, Debug, PartialEq)]
enum Names {
    John,
    #[arg(name = "renamed")]
    Marco,
    ComposedName
}

#[test]
fn test_enum_derive_parsing() {
    let args = vec![
        "--name".to_string(),
        "composed-name".to_string(),
    ];

    let result_1 = Args::parse_args(&args).unwrap();
    assert_eq!(result_1.name, Names::ComposedName);

    let result_2 = Args::parse_str("--name composed-name").unwrap();
    assert_eq!(result_1, result_2);
}