use crate::lexing::{chars, position, position::Position, syntax_error::SyntaxError, token::Token};
use std::iter::Peekable;
use std::str::Chars;

struct Lexer<'a> {
    input: &'a str,
    current: Peekable<Chars<'a>>,
    position: Position,
    errors: Vec<SyntaxError>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            input,
            current: input.chars().peekable(),
            position: position::START,
            errors: Vec::new(),
        }
    }

    fn lex_token(&mut self) -> Token {
        self.skip_whitespace();
        panic!()
    }

    fn skip_whitespace(&mut self) {
        self.skip_while(chars::is_whitespace)
    }

    fn skip_while<P>(&mut self, predicate: P)
    where
        P: Fn(char) -> bool,
    {
        while predicate(self.peek()) {
            self.advance();
        }
    }

    fn peek(&mut self) -> char {
        let cur: Option<&char> = self.current.peek();
        *cur.unwrap_or(&chars::NULL)
    }

    fn peek_char(&mut self, ch: char) -> bool {
        self.peek() == ch
    }

    fn advance(&mut self) {
        self.position = match self.current.next() {
            Some(chars::LINE_FEED) => self.position.next_line(),
            Some(chars::CARRIAGE_RETURN) => {
                // handle windows line endings
                if self.peek_char(chars::LINE_FEED) {
                    self.current.next();
                }
                self.position.next_line()
            }
            _ => self.position.next_column(),
        }
    }
}
