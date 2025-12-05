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
            (Token::Pipe, Token::Pipe) => true,
            _ => false,
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut chars = input.chars().peekable();

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
            ' ' | '\t' | '\n' if !cur.is_empty() => {
                push_token(&mut tokens, cur.clone());
                cur.clear();
            }
            ' ' | '\t' | '\n' => {
            }
            '"' | '\'' => {
                let quote = ch;
                let mut collected = String::new();
                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    if next_ch == '\\' {
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
                expanded_tokens.push(">".to_string());
                prev_token = Some(Token::RedirOut);
            }
            Token::RedirIn => {
                expanded_tokens.push("<".to_string());
                prev_token = Some(Token::RedirIn);
            }
            Token::Background => {
                expanded_tokens.push("&".to_string());
                prev_token = Some(Token::Background);
            }
        }
    }
    expanded_tokens
}

fn resolve_path(s: &str) -> String {
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
