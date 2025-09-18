use core::fmt::Write;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::arg::arg_info::ArgInfo;
use crate::subcommand::SubcommandInfo;

pub fn get_help(out: &mut String, name: Option<&String>, args: &Vec<ArgInfo>, subcommands: &Vec<SubcommandInfo>) {
    if let Some(name) = name {
        writeln!(out, "Usage: {} [OPTIONS] [SUBCOMMAND]", name).unwrap();
    }
    else {
        writeln!(out, "Usage: [OPTIONS] [SUBCOMMAND]").unwrap();
    }

    writeln!(out).unwrap();
    writeln!(out, "Options:").unwrap();
    for arg in args {
        let mut line = String::new();

        if let Some(short) = arg.short {
            line.push('-');
            line.push(short);
            if arg.long.is_some() {
                line.push_str(", ");
            }
        }
        if let Some(long) = &arg.long {
            line.push_str("--");
            line.push_str(long);
        }

        if let Some(help) = &arg.help {
            line.push_str(&format!("\t\t\t{}", help));
        }

        writeln!(out, "  {}", line).unwrap();
    }

    if !subcommands.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "Commands:").unwrap();
        for sub in subcommands {
            let mut line = sub.name.clone();
            if let Some(help) = &sub.about {
                line.push_str(&format!("\t\t\t{}", help));
            }
            writeln!(out, "  {}", line).unwrap();
        }
    }
}