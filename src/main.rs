#![allow(dead_code)]

use crate::lexing::lexer::Lexer;
use crate::parsing::parse_tree::ParseTree;
use crate::parsing::parse_tree_visitor::visit_post_order;
use crate::parsing::parser::Parser;
use crate::utils::syntax_error::SyntaxError;
extern crate csv;
use csv::Reader;
use std::env;
use std::error::Error;
use std::fs;

mod lexing;
mod parsing;
mod utils;

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
        // println!("{:#?}", tree);
    } else {
        println!("{:#?}", errors[0]);
    }
}

fn process_query(query: &str) {
    if !lex_and_dump(query) {
        parse(query);
    }
}

fn read_queries_from_csv(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut result = Vec::new();
    let mut rdr = Reader::from_path(path)?;
    for record_result in rdr.records() {
        let record = record_result?;
        if let Some(field) = record.get(1) {
            result.push(field.to_string());
        }
    }
    Ok(result)
}

fn process_csv(path: &str) -> Result<(), Box<dyn Error>> {
    println!("{}", path);
    let queries = read_queries_from_csv(path)?;
    let mut count = 0;
    for query in &queries {
        // println!("{}", query);
        process_query(&query);
        print!(".");

        count += 1;
        if count % 80 == 0 {
            println!("{}", count);
        }
    }
    Ok(())
}

fn read_and_parse_files(file_names: &[String]) {
    for filename in file_names {
        println!("{}", filename);
        let read_result = fs::read_to_string(filename);
        match read_result {
            Ok(contents) => process_query(&contents),
            Err(e) => println!("Error reading file {}", e),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing file name");
        return;
    }
    if &args[1] == "--csv" {
        if args.len() < 3 {
            println!("Missing file name");
            return;
        }
        process_csv(&args[2]);
    } else {
        read_and_parse_files(&args[1..]);
    }
}
