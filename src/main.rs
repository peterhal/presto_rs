#![allow(dead_code)]

use crate::lexing::lexer::Lexer;
use crate::parsing::parser::Parser;
use std::env;
use std::fs;

mod lexing;
mod parsing;

fn lex_and_dump(contents: &str) {
    let mut lexer = Lexer::new(contents);
    loop {
        println!("{}", lexer.lex_token());
        if lexer.at_end() {
            break;
        }
    }
}

fn parse(contents: &str) {
    let mut parser = Parser::new(contents);
    let tree = parser.parse_query();
    println!("{:?}", tree);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing file name");
        return;
    }

    let filename = &args[1];
    let read_result = fs::read_to_string(filename);
    match read_result {
        Ok(contents) => {
            lex_and_dump(&contents);
            parse("WITH RECURSIVE");
            println!("Hello world!\n{}", contents);
        }
        Err(e) => println!("Error reading file {}", e),
    }
}
