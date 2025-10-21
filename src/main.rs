mod interp;
mod parse;

use interp::interp;
use parse::parse_expression;
use parse::tokenize;
use std::collections::HashMap;
use std::fmt;
use std::io;

#[derive(Debug, PartialEq)]
enum Token {
    Int(isize),
    Bool(bool),
    Str(String),
    Symbol(String),
    Plus,
    Concat,
    LessThan,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Equal,
    If,
    Else,
    Let,
    Fn,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Int(n) => write!(f, "{n}"),
            Token::Bool(b) => write!(f, "{b}"),
            Token::Str(s) => write!(f, "\"{s}\""),
            Token::Symbol(s) => write!(f, "{s}"),
            Token::Plus => write!(f, "+"),
            Token::Concat => write!(f, "++"),
            Token::LessThan => write!(f, "<"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Equal => write!(f, "="),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Let => write!(f, "let"),
            Token::Fn => write!(f, "fn"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Exp {
    Int(isize),
    Bool(bool),
    Str(String),
    Var(String),
    Plus { left: Box<Exp>, right: Box<Exp> },
    Concat { left: Box<Exp>, right: Box<Exp> },
    LessThan { left: Box<Exp>, right: Box<Exp> },
    Cnd { tst: Box<Exp>, thn: Box<Exp>, els: Box<Exp> },
    Let1 { var: String, value: Box<Exp>, body: Box<Exp> },
    Lam { var: String, body: Box<Exp> },
    App { fun: Box<Exp>, arg: Box<Exp> },
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Exp::Int(n) => write!(f, "{n}"),
            Exp::Bool(b) => write!(f, "{b}"),
            Exp::Str(s) => write!(f, "\"{s}\""),
            Exp::Var(v) => write!(f, "{v}"),
            Exp::Plus { left, right } => write!(f, "(+ {left} {right})"),
            Exp::Concat { left, right } => write!(f, "(++ {left} {right})"),
            Exp::LessThan { left, right } => write!(f, "(< {left} {right})"),
            Exp::Cnd { tst, thn, els } => write!(f, "(if {tst} {thn} {els})"),
            Exp::Let1 { var, value, body } => {
                write!(f, "(let {var} {value} {body})")
            }
            Exp::Lam { var, body } => { write!(f, "(fn ({var}) {body})") }
            Exp::App { fun, arg } => write!(f, "({fun} {arg})"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Value {
    Int(isize),
    Bool(bool),
    Str(String),
    Fun { var: String, body: Box<Exp>, nv: Env },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Fun { var, body, nv } => write!(
                f,
                "closure((lambda ({var}) {body}), {nv:?})"
            ),
        }
    }
}

type Env = HashMap<String, Value>;

fn main() {
    let empty_nv = Env::new();

    loop {
        // print a prompt
        println!("\nPlease enter an expression:");
        let mut tokens = Vec::new();

        loop {
            // read a line of input, quit on ctrl-d and skip empty lines
            let mut input = String::new();
            let len =
                io::stdin().read_line(&mut input).expect("Failed to read line");
            if len == 0 {
                return;
            }
            if input.trim().is_empty() {
                continue;
            }

            // tokenize
            match tokenize(input.trim()) {
                Ok(new_tokens) => {
                    tokens.extend(new_tokens);
                }
                Err(msg) => {
                    println!("Tokenizer error: {msg}");
                    continue;
                }
            };

            // scan the token list and count total nesting level
            // we finish if we are at zero
            let mut count = 0;
            for elt in &tokens {
                match elt {
                    Token::LeftParen => count += 1,
                    Token::RightParen => count -= 1,
                    Token::LeftBrace => count += 1,
                    Token::RightBrace => count -= 1,
                    _ => {}
                }
            }
            if count == 0 {
                break;
            }
        }

        print!("tokens: [");
        let mut sep = "";
        for t in &tokens {
            print!("{sep}{t}");
            sep = ", ";
        }
        println!("]");

        // parse
        let ast = match parse_expression(&tokens) {
            Ok(ast) => ast,
            Err(msg) => {
                println!("Parse error: {msg}");
                continue;
            }
        };
        println!("ast   : {ast}");

        // evaluate
        let v = match interp(&ast, &empty_nv) {
            Ok(v) => v,
            Err(msg) => {
                println!("Runtime error: {msg}");
                continue;
            }
        };
        println!("result: {v}");
    }
}
