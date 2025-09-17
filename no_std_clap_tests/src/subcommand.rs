use alloc::string::{String, ToString};
use alloc::vec;
use no_std_clap_core::parser::Parser;
use no_std_clap_macros::{Args, Parser, Subcommand};

#[derive(Parser, Debug, PartialEq)]
#[clap(name = "myapp", version = "1.0", about = "A sample CLI app")]
struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Commands {
    #[command(about = "Add a new item")]
    Add(AddArgs),

    #[command(about = "Remove an item")]
    Remove(RemoveArgs),

    #[command(name = "list", about = "List all items")]
    List,
}


#[derive(Args, Debug, PartialEq)]
struct AddArgs {
    #[arg(short, long, required)]
    name: String,

    #[arg(short, long)]
    force: bool,
}

#[derive(Args, Debug, PartialEq)]
struct RemoveArgs {
    #[arg(short, long, required)]
    name: String,

    #[arg(short, long)]
    recursive: bool,
}

#[test]
fn test_subcommand_add() {
    let args = vec![
        "--verbose".to_string(),
        "true".to_string(),
        "add".to_string(),
        "--name".to_string(),
        "test_item".to_string(),
        "--force".to_string(),
    ];

    let cli = Cli::parse_args(&args).unwrap();
    assert!(cli.verbose);

    match cli.command {
        Some(Commands::Add(add_args)) => {
            assert_eq!(add_args.name, "test_item");
            assert!(add_args.force);
        }
        _ => panic!("Expected Add subcommand"),
    }
}

#[test]
fn test_subcommand_remove() {
    let args = vec![
        "remove".to_string(),
        "--name".to_string(),
        "old_item".to_string(),
        "--recursive".to_string(),
    ];

    let cli = Cli::parse_args(&args).unwrap();
    assert!(!cli.verbose);

    match cli.command {
        Some(Commands::Remove(remove_args)) => {
            assert_eq!(remove_args.name, "old_item");
            assert!(remove_args.recursive);
        }
        _ => panic!("Expected Remove subcommand"),
    }
}

#[test]
fn test_subcommand_list() {
    let args = vec![
        "list".to_string(),
    ];

    let cli = Cli::parse_args(&args).unwrap();
    if !matches!(cli.command, Some(Commands::List)) {
        panic!("Expected list subcommand");
    }
}

#[test]
fn test_no_subcommand() {
    let args = vec![
        "--verbose".to_string(),
    ];

    let cli = Cli::parse_args(&args).unwrap();
    assert!(cli.verbose);
    assert_eq!(cli.command, None);
}
