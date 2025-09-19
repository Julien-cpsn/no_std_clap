use core::fmt::Write;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::arg::arg_info::ArgInfo;
use crate::subcommand::SubcommandInfo;

pub fn get_help(out: &mut String, name: Option<&String>, args: &Vec<ArgInfo>, global_args: &Vec<ArgInfo>,subcommands: &Vec<SubcommandInfo>) {
    if let Some(name) = name {
        write!(out, "Usage: {}", name).unwrap();
    }

    let positional_args: Vec<&ArgInfo> = args.iter().filter(|a| a.short.is_none() && a.long.is_none()).collect();
    let flag_args: Vec<&ArgInfo> = args.iter().filter(|a| a.short.is_some() || a.long.is_some()).collect();

    for arg in &positional_args {
        write!(out, " <{}>", arg.name.to_uppercase()).unwrap();
    }

    if name.is_some() && (!flag_args.is_empty() || !global_args.is_empty()) {
        write!(out, " [OPTIONS]").unwrap()
    }

    if name.is_some() && !subcommands.is_empty() {
        write!(out, " [SUBCOMMAND]").unwrap();
    }

    if name.is_some() && (!positional_args.is_empty() || !flag_args.is_empty() || !subcommands.is_empty()) {
        writeln!(out).unwrap();
        writeln!(out).unwrap();
    }

    if !positional_args.is_empty() {
        writeln!(out, "Arguments:").unwrap();
        for arg in positional_args {
            let mut line = String::new();

            write!(line, "{}", arg.name.to_uppercase()).unwrap();

            if let Some(help) = &arg.help {
                line.push_str(&format!("\t\t\t{}", help));
            }

            writeln!(out, "  {}", line).unwrap();
        }
    }

    if !flag_args.is_empty() || !global_args.is_empty() {
        writeln!(out, "Options:").unwrap();
        for arg in flag_args {
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

        for arg in global_args {
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
    }

    if !subcommands.is_empty() {
        if !args.is_empty() {
            writeln!(out).unwrap();
        }

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