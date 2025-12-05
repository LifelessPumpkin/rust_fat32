use std::{env, path::Path};
// use std::io::{stdin, stdout, Write};
use crate::commands::*;


pub enum Token {
    Word(String),
    Argument(String),
    EnvVar(String),
    Pipe,
    RedirOut,
    RedirIn,
    Background,
    Tilde(String),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare only the discriminant (variant name) for Pipe
            (Token::Pipe, Token::Pipe) => true,
            // Compare RedirOut and RedirIn by their inner String values
            // (Token::RedirOut(s1), Token::RedirOut(s2)) => s1 == s2,
            // (Token::RedirIn(s1), Token::RedirIn(s2)) => s1 == s2,
            // All other combinations are not equal
            _ => false,
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    // Split using whitespace, then match on special chars
    for part in input.split_whitespace() {
        match part {
            "|" => tokens.push(Token::Pipe),
            ">" => tokens.push(Token::RedirOut),
            "<" => tokens.push(Token::RedirIn),
            "&" => tokens.push(Token::Background),
            // Only need to handle the ~ and ~/ case
            t if t.starts_with('~') || t.starts_with("~/") => tokens.push(Token::Tilde(part.to_string())), 
            p if p.starts_with('$') => tokens.push(Token::EnvVar(p[1..].to_string())),
            a if a.starts_with('-') => tokens.push(Token::Argument(part.to_string())),
            _ => tokens.push(Token::Word(part.to_string())),
        }
    }
    tokens
}

pub fn expand_tokens(tokens: Vec<Token>) -> Vec<String> {
    let mut expanded_tokens: Vec<String> = Vec::new();
    let mut prev_token: Option<Token> = None;

    for token in tokens {
        match token {
            Token::EnvVar(name) => {
                if let Ok(val) = env::var(&name) {
                    expanded_tokens.push(val);
                    prev_token = Some(Token::EnvVar(name));
                } else {
                    expanded_tokens.push(String::new());
                    prev_token = Some(Token::EnvVar(name));
                }
            }
            Token::Tilde(s) => {
                let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
                if s == "~" {
                    expanded_tokens.push(home);
                } else if let Some(rest) = s.strip_prefix("~/") {
                    let full = format!("{}/{}", home, rest);
                    expanded_tokens.push(full);
                } else {
                    expanded_tokens.push(s.clone());
                }
                prev_token = Some(Token::Tilde(s));
            }

            Token::Word(s) => {
                // If this is the first line or it follows a pipe, I need to search PATH for the executable
                // If it follows a redirection, it's a filename, so just add it as an argument
                // Otherwise, it's just an argument
                if prev_token.is_none() || prev_token == Some(Token::Pipe) {
                    let program = resolve_path(&s);
                    expanded_tokens.push(program);
                    prev_token = Some(Token::Word(s));
                    continue;
                } else {
                    expanded_tokens.push(s);
                    prev_token = Some(Token::Word(String::new()));
                    continue;
                }
            }
            Token::Argument(s) => {
                expanded_tokens.push(s);
                prev_token = Some(Token::Argument(String::new()));
            }
            Token::Pipe => {
                expanded_tokens.push("|".to_string());
                prev_token = Some(Token::Pipe);
            }
            Token::RedirOut => {
                // If it's a redirection, the next token should be a filename
                expanded_tokens.push(">".to_string());
                prev_token = Some(Token::RedirOut);
            }
            Token::RedirIn => {
                // If it's a redirection, the next token should be a filename
                expanded_tokens.push("<".to_string());
                prev_token = Some(Token::RedirIn);
            }
            Token::Background => {
                expanded_tokens.push("&".to_string());
                prev_token = Some(Token::Background);
            }
        }
    }
    // For debugging purposes
    // println!("Expanded Tokens: ");
    // for t in expanded_tokens.iter() {
    //     println!("{} ", t.to_str().unwrap());
    // }
    // println!();
    expanded_tokens
}

// Maybe I should also resolve built-in commands here?
fn resolve_path(s: &str) -> String {
    // First check if its a built-in command
    if is_built_in(s) {
        return s.to_string();
    }
    if let Ok(path) = env::var("PATH") {
        let paths: Vec<&str> = path.split(':').collect();

        for path in paths.iter() {
            let full_path = &format!("{}/{}", path, s);

            if Path::new(&full_path).exists() {
                // let program = CString::new(full_path.as_str()).unwrap();
                
                return full_path.to_string();
            }
        }
    } else {
        println!("PATH environment variable is not set.");
    }
    s.to_string()
}
