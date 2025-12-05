use crate::commands::*;
use crate::{
    models::ShellCore, 
    parser::{expand_tokens, tokenize}};


struct CommandPart {
    program: String,
    args: Vec<String>,
    redir_in: Option<String>,
    redir_out: Option<String>,
    direction: Option<Direction>,
    background: bool,
    parse_error: Option<&'static str>, 
}

enum Direction {
    Pipe,
}

impl PartialEq for Direction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Direction::Pipe, Direction::Pipe) => true,
        }
    }
}

pub fn execute_command(command: &str, shell: &mut ShellCore) {
    // Phase 1: Tokenization and Expansion
    let tokens: Vec<_> = tokenize(command);
    let expanded_tokens: Vec<String> = expand_tokens(tokens);

    // Phase 2: Interpretation and Execution
    let commands: Vec<CommandPart> = interpret_tokens(expanded_tokens);
    execute(commands, shell);
}

fn interpret_tokens(tokens: Vec<String>) -> Vec<CommandPart> {
    let mut command_parts: Vec<CommandPart> = Vec::new();
    let mut current_part: Option<CommandPart> = None;

    let mut tokens_iter = tokens.iter().peekable();
    while let Some(t) = tokens_iter.next() {
        // The first token is always the program
        // The rest are arguments until I hit a special token

        if current_part.is_none() {
            current_part = Some(CommandPart {
                program: t.clone(),
                args: Vec::new(),
                redir_in: None,
                redir_out: None,
                direction: None,
                background: false,
                parse_error: None,
            });
        } else {
            current_part.as_mut().unwrap().args.push(t.clone());
        }

        if t == "|" {
            current_part.as_mut().unwrap().args.pop();
            current_part.as_mut().unwrap().direction = Some(Direction::Pipe);
            command_parts.push(current_part.take().unwrap());
            current_part = None;
        } else if t == ">" {
            current_part.as_mut().unwrap().args.pop(); // remove ">" from args
            if let Some(next_token) = tokens_iter.next() {
                let filename = next_token.to_string();
                current_part.as_mut().unwrap().redir_out = Some(filename);
            } else {
                current_part.as_mut().unwrap().parse_error = Some("missing output file after '>'");
            }
        } else if t == "<" {
            current_part.as_mut().unwrap().args.pop(); // remove "<" from args
            if let Some(next_token) = tokens_iter.next() {
                let filename = next_token.to_string();
                current_part.as_mut().unwrap().redir_in = Some(filename);
            } else {
                current_part.as_mut().unwrap().parse_error = Some("missing input file after '<'");
            }
        } else if t == "&" {
            current_part.as_mut().unwrap().background = true;
            current_part.as_mut().unwrap().args.pop();
            command_parts.push(current_part.take().unwrap());
            current_part = None;
        } else {
            if t == tokens.last().unwrap() && current_part.is_some() {
                command_parts.push(current_part.take().unwrap());
            }
        }
    }
    if let Some(final_part) = current_part.take() {
            command_parts.push(final_part);
        }
    
    // For debugging purposes
    // println!("Interpreted Command Parts: ");
    // for part in command_parts.iter() {
    //     println!("Program: {}", part.program.to_str().unwrap());
    //     for arg in part.args.iter() {
    //         println!("Arg: {}", arg.to_str().unwrap());
    //     }
    //     if let Some(dir) = &part.direction {
    //         match dir {
    //             Direction::Pipe => println!("Direction: Pipe"),
    //         }
    //     } else {
    //         println!("Direction: None");
    //     }
    //     if let Some(redir_out) = &part.redir_out {
    //         println!("Redir Out: {}", redir_out);
    //     }
    //     if let Some(redir_in) = &part.redir_in {
    //         println!("Redir In: {}", redir_in);
    //     }
    //     println!("Background: {}", part.background);
    // }
    command_parts
}

fn execute(commands: Vec<CommandPart>, shell: &mut ShellCore) {
    for part in commands.iter() {
        if is_built_in(&part.program) {
            execute_built_in(&part.program, shell, &part.args);
        } 
    }
}


