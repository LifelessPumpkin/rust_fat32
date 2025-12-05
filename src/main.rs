use std::env::{args};
use std::fs::{OpenOptions};
use std::io::{Write, stdin, stdout};
use crate::executor::execute_command;
use crate::models::ShellCore;


mod models;
mod core;
mod parser;
mod commands;
mod executor;
mod builtins;
    
fn main() {
    let args: Vec<String> = args().collect();
    if args.len() > 2 || args.len() < 2 {
        eprintln!("This shell takes exactly one argument: the image name.\nUsage: rust_fat32 <image_name>");
        std::process::exit(1);
    }
    let image = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(&args[1])
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open image file '{}': {}", args[1], e);
            std::process::exit(1);
        }
    };

    let mut shell = crate::models::ShellCore::new(image);
    loop {
        create_prompt(&shell);

        let mut input: String = String::new();
        stdin().read_line(&mut input).unwrap();
        let command = input.trim();

        if command.is_empty() {
            continue;
        }

        execute_command(command, &mut shell);


    }
}

fn create_prompt(shell: &ShellCore) {

    let image = args().nth(1).unwrap();

    let path = shell.cwd_path.clone();

    print!("{}{}>",image, path);
    match stdout().flush() {
        Ok(res) => res,
        Err(_) => print!("Error Flushing")
    }    

}
