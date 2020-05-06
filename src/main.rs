#![allow(dead_code)]

use crate::lexing::Lexer;
use crate::parsing::parse_statement;
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

fn parse(contents: &str) {
    let (_tree, errors) = parse_statement(contents);
    if errors.is_empty() {
        println!("{:#?}", _tree);
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

fn process_csv(path: &str) -> Result<(), Box<dyn Error>> {
    println!("{}", path);
    let mut count = 0;
    let mut rdr = Reader::from_path(path)?;
    for record_result in rdr.records() {
        let record = record_result?;
        if let Some(field) = record.get(1) {
            process_query(&field.to_string());
            if count % 100 == 0 {
                print!("{} ", count);
            }
            print!(".");
            count += 1;
            if count % 100 == 0 {
                println!("");
            }
        }
    }
    if count % 100 != 0 {
        println!("");
    }
    println!("OK");
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
    read_and_parse_files(&["private/test.sql".to_owned()]);
    //   let args: Vec<String> = env::args().collect();
    //    if args.len() < 2 {
    //        println!("Missing file name");
    //        return;
    //    }
    //    if &args[1] == "--csv" {
    //        if args.len() < 3 {
    //            println!("Missing file name");
    //            return;
    //        }
    //        let _result = process_csv(&args[2]);
    //    } else {
    //        read_and_parse_files(&args[1..]);
    //    }
}
