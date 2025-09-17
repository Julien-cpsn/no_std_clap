# `no_std` clap

`no_std` compliant clone of rust clap (command line argument parser) with minimal functionalities.

> [!WARNING]
> You still need `alloc`

## Usage

> [!NOTE]
> You can either use `MyArgs::parse_str(&str)` or `MyArgs::parse_args(Vec<&str>)`.

### With derive

**Basic**

```rust
use alloc::string::String;
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

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    list: Vec<u8>,

    // You can also add `default_value = "Default"`
    #[arg(long)]
    optional: Option<String>,

    #[arg(skip)]
    computed: String, // Will use Default::default()
}

fn my_function() {
    let result = TestArgs::parse_str("--name test --count 42 --verbose true --list 5").unwrap();
}
```

**With subcommand**

```rust
use alloc::string::String;
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

fn my_function() {
    let cli = Cli::parse_str("--verbose true add --name test_item --force").unwrap();
    
    if let Some(subcommand) = cli.command {
        match subcommand {
            Commands::Add(add_args) => {},
            Commands::Remove(remove_args) => {},
            Commands::List => {},
        }
    }
}
```

### Without derive

**Basic**

```rust
use alloc::string::String;
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
        let cmd = Command::new("test")
            .arg(ArgInfo::new("name").long("name").short('n').required())
            .arg(ArgInfo::new("count").long("count").short('c').required())
            .arg(ArgInfo::new("verbose").long("verbose").short('v'))
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
}

fn my_function() {
    let result = TestArgs::parse_str("--name test --count 42 --verbose true --list 5").unwrap();
}
```