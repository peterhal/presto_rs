use crate::lexing::{ token::Token};
use std::str::Chars;

struct Lexer<'a> {
    input: &'a str,
    current: Chars<'a>,
}

impl<'a> Lexer<'a> {
    fn lex_token(&mut self) -> Token {
        panic!()
    }
}
