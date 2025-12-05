use crate::{
    builtins::*,
    models::ShellCore};

pub fn is_built_in(command: &str) -> bool {
    match command {
        "info" | "exit" | "cd" | "ls" | "open" | "close" | "lsof" | "lseek" | "read" | "mkdir" | "creat" => true,
        _ => false,
    }
}

pub fn execute_built_in(command: &str, shell: &mut ShellCore, args: &[String]) {
    match command {
        "info" => info::info(&shell.vol.bpb),
        "exit" => exit::exit(),
        "cd" => cd::cd(args.get(0).map(|s| s.as_str()).unwrap_or(""), shell),
        "ls" => ls::ls(shell),
        "open" => open::open(args.get(0).map(|s| s.as_str()).unwrap_or(""),
         args.get(1).map(|s| s.as_str()).unwrap_or("r"), shell),
        "close" => close::close(args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0), shell),
        "lsof" => lsof::lsof(shell),
        "lseek" => lseek::lseek(args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0), args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0), shell),
        "read" => read::read(args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0), args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0), shell),  
        "mkdir" => mkdir::mkdir(args.get(0).map(|s| s.as_str()).unwrap_or(""), shell),
        "creat" => creat::creat(args.get(0).map(|s| s.as_str()).unwrap_or(""), shell),
        _ => eprintln!("Unknown built-in command: {}", command),
    }
}