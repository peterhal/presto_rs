use crate::lexing::{
    chars, position, position::Position, syntax_error::SyntaxError, text_range::TextRange,
    token::Token, token_kind::TokenKind,
};
use std::str::Chars;

#[derive(Clone, Copy, Debug)]
pub struct LexerPosition<'a> {
    source: &'a str,
    position: Position,
    // index into source in bytes
    index: usize,
}

impl<'a> LexerPosition<'a> {
    fn new(source: &'a str) -> LexerPosition<'a> {
        LexerPosition {
            source,
            position: position::START,
            index: 0,
        }
    }

    fn skip_while<P>(&mut self, predicate: P)
    where
        P: Fn(char) -> bool,
    {
        while predicate(self.peek()) {
            self.next();
        }
    }

    fn chars(&self) -> Chars<'a> {
        // TODO: from_utf8_unchecked
        if let Ok(s) = std::str::from_utf8(self.source.as_bytes()) {
            s[self.index..].chars()
        } else {
            panic!("bad str")
        }
    }

    fn peek(&self) -> char {
        self.chars().next().unwrap_or(chars::NULL)
    }

    fn peek_offset(&self, offset: i32) -> char {
        assert!(offset >= 0);

        let mut clone = self.clone();
        while offset > 0 {
            clone.next();
        }
        clone.peek()
    }

    fn peek_char(&self, ch: char) -> bool {
        self.peek() == ch
    }

    fn peek_char_offset(&self, ch: char, offset: i32) -> bool {
        self.peek_offset(offset) == ch
    }

    fn at_end(&self) -> bool {
        self.peek_char(chars::NULL)
    }

    fn advance_index_of_char(&mut self, ch: Option<char>) {
        if let Some(ch) = ch {
            self.index += ch.len_utf8();
        }
    }

    fn next(&mut self) -> char {
        let ch = self.chars().next();
        self.advance_index_of_char(ch);
        self.position = match ch {
            Some(chars::LINE_FEED) => self.position.next_line(),
            Some(chars::CARRIAGE_RETURN) => {
                // handle windows line endings
                if self.peek_char(chars::LINE_FEED) {
                    self.advance_index_of_char(self.chars().next());
                }
                self.position.next_line()
            }
            Some(_) => self.position.next_column(),
            None => self.position,
        };
        ch.unwrap_or(chars::NULL)
    }

    fn get_range(&self, end: &LexerPosition) -> TextRange {
        TextRange {
            start: self.position,
            end: end.position,
        }
    }

    fn get_text(&self, end: &LexerPosition) -> &'a str {
        assert!(end.index >= self.index);
        &self.source[self.index..end.index]
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    position: LexerPosition<'a>,
    errors: Vec<SyntaxError>,
}

// All the non-language specific infrastructure goes here
impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            input,
            position: LexerPosition::new(input),
            errors: Vec::new(),
        }
    }

    fn mark(&self) -> LexerPosition<'a> {
        self.position
    }

    fn peek(&mut self) -> char {
        self.position.peek()
    }

    fn eat_opt(&mut self, ch: char) -> bool {
        if self.peek_char(ch) {
            self.next();
            true
        } else {
            false
        }
    }

    fn peek_offset(&mut self, offset: i32) -> char {
        self.position.peek_offset(offset)
    }

    fn peek_char(&mut self, ch: char) -> bool {
        self.position.peek_char(ch)
    }

    fn peek_char_offset(&mut self, ch: char, offset: i32) -> bool {
        self.position.peek_char_offset(ch, offset)
    }

    pub fn at_end(&mut self) -> bool {
        self.position.at_end()
    }

    fn next(&mut self) -> char {
        self.position.next()
    }

    fn create_token(&self, start: &LexerPosition<'a>, kind: TokenKind) -> Token<'a> {
        Token {
            kind,
            range: self.get_range(start),
            value: self.get_text(start),
            errors: Vec::new(),
            leading_comments: Vec::new(),
            trailing_comments: Vec::new(),
        }
    }

    fn get_range(&self, start: &LexerPosition<'a>) -> TextRange {
        start.get_range(&self.position)
    }

    fn get_text(&self, start: &LexerPosition<'a>) -> &'a str {
        start.get_text(&self.position)
    }
}

// Language specific lexing goes here:
impl<'a> Lexer<'a> {
    fn skip_whitespace(&mut self) {
        self.position.skip_while(chars::is_whitespace)
    }

    pub fn lex_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        let start = self.mark();
        if self.at_end() {
            self.create_token(&start, TokenKind::END_OF_FILE)
        } else {
            panic!("TODO")
        }
    }
}
