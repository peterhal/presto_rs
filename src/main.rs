#![allow(dead_code)]

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
        Ok(contents) => println!("Hello world!\n{}", contents),
        Err(e) => println!("Error reading file {}", e),
    }
}
