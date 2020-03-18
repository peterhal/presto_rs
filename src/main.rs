#![allow(dead_code)]

use crate::lexing::lexer::Lexer;
use std::env;
use std::fs;

mod lexing;

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
            let mut lexer = Lexer::new(&contents);
            loop {
                println!("{}", lexer.lex_token());
                if lexer.at_end() {
                    break;
                }
            }
            println!("Hello world!\n{}", contents)
        }
        Err(e) => println!("Error reading file {}", e),
    }
}
