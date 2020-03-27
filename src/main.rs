#![allow(dead_code)]

use crate::lexing::lexer::Lexer;
use crate::lexing::syntax_error::SyntaxError;
use crate::parsing::parse_tree::ParseTree;
use crate::parsing::parse_tree_visitor::visit_post_order;
use crate::parsing::parser::Parser;
use std::env;
use std::fs;

mod lexing;
mod parsing;

fn lex_and_dump(contents: &str) -> bool {
    let mut lexer = Lexer::new(contents);
    let mut had_error = false;
    loop {
        let token = lexer.lex_token();
        let errors = token.errors;
        if !errors.is_empty() {
            println!("{:#?}", errors);
            had_error = true;
        }
        if lexer.at_end() {
            break;
        }
    }
    had_error
}

fn errors_of_tree<'a>(tree: &'a ParseTree<'a>) -> Vec<&'a SyntaxError> {
    let mut errors: Vec<&'a SyntaxError> = Vec::new();
    let mut visit = |tree: &'a ParseTree<'a>| match tree {
        ParseTree::Token(tree) => {
            for error in &tree.token.errors {
                errors.push(&error)
            }
        }
        ParseTree::Error(error) => errors.push(&error.error),
        _ => (),
    };
    visit_post_order(tree, &mut visit);
    errors
}

fn parse(contents: &str) {
    let mut parser = Parser::new(contents);
    let tree = parser.parse_statement();
    let errors = errors_of_tree(&tree);
    if errors.is_empty() {
        //    println!("{:#?}", tree);
    } else {
        println!("{:#?}", errors[0]);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing file name");
        return;
    }

    for filename in &args[1..] {
        println!("{}", filename);
        let read_result = fs::read_to_string(filename);
        match read_result {
            Ok(contents) => {
                if !lex_and_dump(&contents) {
                    parse(&contents);
                }
            }
            Err(e) => println!("Error reading file {}", e),
        }
    }
}
