use alloc::string::{String, ToString};
use alloc::vec;
use no_std_clap_core::parser::Parser;
use no_std_clap_macros::{Args, Parser, Subcommand};

#[derive(Parser, Debug, PartialEq)]
#[clap(name = "myapp", version = "1.0", about = "A sample CLI app")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, count, global)]
    verbose: usize
}

#[derive(Subcommand, Debug, PartialEq)]
enum Commands {
    /// Add a new item
    Add(AddArgs),

    #[command(subcommand, about = "Remove an item")]
    Remove(RemoveCommand),

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

#[derive(Subcommand, Debug, PartialEq)]
enum RemoveCommand {
    One,
    All(RemoveAllCommand),
}

#[derive(Args, Debug, PartialEq)]
struct RemoveAllCommand {
    #[arg(short, long, required)]
    pub name: String,

    #[arg(short, long)]
    pub recursive: bool,
}


#[test]
fn test_subcommand_add() {
    let args = vec![
        "add".to_string(),
        "--name".to_string(),
        "test_item".to_string(),
        "--force".to_string(),
        "-vvv".to_string(),
    ];

    let cli = Cli::parse_args(&args).unwrap();
    assert_eq!(cli.verbose, 3);

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
        "all".to_string(),
        "--name".to_string(),
        "old_item".to_string(),
        "--recursive".to_string(),
    ];

    let cli = Cli::parse_args(&args).unwrap();
    assert_eq!(cli.verbose, 0);

    match cli.command {
        Some(Commands::Remove(RemoveCommand::All(remove_all_command))) => {
            assert_eq!(remove_all_command.name, "old_item");
            assert!(remove_all_command.recursive);
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
        "-v".to_string(),
    ];

    let cli = Cli::parse_args(&args).unwrap();
    assert_eq!(cli.command, None);
    assert_eq!(cli.verbose, 1);
}