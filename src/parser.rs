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
    // Tokenize by iterating characters so quoted strings remain a single token.
    let mut cur = String::new();
    let mut chars = input.chars().peekable();

    // helper to push a completed token string into tokens with correct variant
    fn push_token(tokens: &mut Vec<Token>, s: String) {
        if s.is_empty() {
            return;
        }
        match s.as_str() {
            "|" => tokens.push(Token::Pipe),
            ">" => tokens.push(Token::RedirOut),
            "<" => tokens.push(Token::RedirIn),
            "&" => tokens.push(Token::Background),
            other => {
                if other.starts_with('~') {
                    tokens.push(Token::Tilde(other.to_string()));
                } else if other.starts_with('$') {
                    tokens.push(Token::EnvVar(other[1..].to_string()));
                } else if other.starts_with('-') {
                    tokens.push(Token::Argument(other.to_string()));
                } else {
                    tokens.push(Token::Word(other.to_string()));
                }
            }
        }
    }

    while let Some(ch) = chars.next() {
        match ch {
            // whitespace separates tokens when not in quotes
            ' ' | '\t' | '\n' if !cur.is_empty() => {
                push_token(&mut tokens, cur.clone());
                cur.clear();
            }
            ' ' | '\t' | '\n' => {
                // skip multiple whitespace
            }
            '"' | '\'' => {
                // quoted string: collect until matching quote, honoring backslash escapes
                let quote = ch;
                let mut collected = String::new();
                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    if next_ch == '\\' {
                        // escape next character if any
                        if let Some(esc) = chars.next() {
                            collected.push(esc);
                        }
                        continue;
                    }
                    if next_ch == quote {
                        break;
                    }
                    collected.push(next_ch);
                }
                // push any pending unquoted prefix + quoted content as a single token
                if !cur.is_empty() {
                    cur.push_str(&collected);
                    push_token(&mut tokens, cur.clone());
                    cur.clear();
                } else {
                    push_token(&mut tokens, collected);
                }
            }
            _ => {
                cur.push(ch);
            }
        }
    }

    if !cur.is_empty() {
        push_token(&mut tokens, cur);
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
