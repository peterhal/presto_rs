use crate::lexing::{
    chars, position, position::Position, syntax_error::SyntaxError, text_range::TextRange,
    token::Token, token_kind::TokenKind,
};
use std::str::Chars;

#[derive(Clone, Debug)]
struct LexerPosition<'a> {
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
            self.advance();
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

    fn peek_char(&self, ch: char) -> bool {
        self.peek() == ch
    }

    fn at_end(&self) -> bool {
        self.peek_char(chars::NULL)
    }

    fn advance_index_of_char(&mut self, ch: Option<char>) {
        if let Some(ch) = ch {
            self.index += ch.len_utf8();
        }
    }

    fn advance(&mut self) {
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
        }
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

struct Lexer<'a> {
    input: &'a str,
    position: LexerPosition<'a>,
    errors: Vec<SyntaxError>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            input,
            position: LexerPosition::new(input),
            errors: Vec::new(),
        }
    }

    fn lex_token(&mut self) -> Token {
        self.skip_whitespace();
        let start = self.position.clone();

        if self.at_end() {
            self.create_token(&start, TokenKind::END_OF_FILE)
        } else {
            panic!()
        }
    }

    fn skip_whitespace(&mut self) {
        self.position.skip_while(chars::is_whitespace)
    }

    fn peek(&mut self) -> char {
        self.position.peek()
    }

    fn peek_char(&mut self, ch: char) -> bool {
        self.position.peek_char(ch)
    }

    fn at_end(&mut self) -> bool {
        self.position.at_end()
    }

    fn advance(&mut self) {
        self.position.advance()
    }

    fn create_token(&self, start: &LexerPosition<'a>, kind: TokenKind) -> Token<'a> {
        Token {
            kind,
            range: start.get_range(&self.position),
            value: start.get_text(&self.position),
            errors: Vec::new(),
            leading_comments: Vec::new(),
            trailing_comments: Vec::new(),
        }
    }
}
