use crate::lexing::{
    chars, comment::Comment, comment::CommentKind, lexer_position::LexerPosition, syntax_error,
    syntax_error::Message, syntax_error::SyntaxError, text_range::TextRange, token::Token,
    token_kind::TokenKind,
};
use std::mem;

pub struct Lexer<'a> {
    input: &'a str,
    position: LexerPosition<'a>,
    comments: Vec<Comment<'a>>,
    errors: Vec<SyntaxError>,
}

// All the non-language specific infrastructure goes here
impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            input,
            position: LexerPosition::new(input),
            errors: Vec::new(),
            comments: Vec::new(),
        }
    }

    fn mark(&self) -> LexerPosition<'a> {
        self.position
    }

    fn peek(&mut self) -> char {
        self.position.peek()
    }

    fn eat(&mut self, ch: char) -> bool {
        if self.eat_opt(ch) {
            true
        } else {
            let actual = self.peek();
            self.add_error(&format!("Expected '{}'; got '{}'", ch, actual));
            false
        }
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

    fn create_token(&mut self, start: &LexerPosition<'a>, kind: TokenKind) -> Token<'a> {
        Token {
            kind,
            range: self.get_range(start),
            value: self.get_text(start),
            errors: Vec::new(),
            leading_comments: mem::replace(&mut self.comments, Vec::new()),
            trailing_comments: Vec::new(),
        }
    }

    fn get_range(&self, start: &LexerPosition<'a>) -> TextRange {
        start.get_range(&self.position)
    }

    fn get_text(&self, start: &LexerPosition<'a>) -> &'a str {
        start.get_text(&self.position)
    }

    fn add_error(&mut self, message: &str) {
        let mut end = self.position.clone();
        end.next();
        self.errors.push(SyntaxError {
            error_code: syntax_error::ERROR_EXPECTED_CHAR,
            messages: vec![Message {
                range: self.position.get_range(&end),
                message: String::from(message),
            }],
        })
    }

    fn create_error(&mut self, start: &LexerPosition<'a>) -> Token<'a> {
        self.create_token(start, TokenKind::Error)
    }

    fn add_comment(&mut self, comment: Comment<'a>) {
        self.comments.push(comment)
    }

    pub fn create_comment(&self, start: &LexerPosition<'a>, kind: CommentKind) -> Comment<'a> {
        Comment {
            kind,
            range: self.get_range(start),
            value: self.get_text(start),
        }
    }

    fn create_and_add_comment(&mut self, start: &LexerPosition<'a>, kind: CommentKind) {
        let comment = self.create_comment(start, kind);
        self.add_comment(comment)
    }
}

// Language specific lexing goes here:
impl<'a> Lexer<'a> {
    fn skip_whitespace(&mut self) {
        self.position.skip_while(chars::is_whitespace)
    }

    fn skip_to_end_of_line(&mut self) {
        loop {
            if self.at_end() {
                break;
            }
            let ch = self.next();
            match ch {
                chars::CARRIAGE_RETURN | chars::LINE_FEED => break,
                _ => (),
            }
        }
    }

    pub fn lex_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        let start = self.mark();
        if self.at_end() {
            self.create_token(&start, TokenKind::EndOfFile)
        } else {
            let ch = self.next();
            match ch {
                // operators and punctuators
                '(' => self.create_token(&start, TokenKind::OpenParen),
                ')' => self.create_token(&start, TokenKind::CloseParen),
                ',' => self.create_token(&start, TokenKind::Comma),
                '.' => self.create_token(&start, TokenKind::Period),
                '<' => match self.peek() {
                    '>' => {
                        self.next();
                        self.create_token(&start, TokenKind::LessGreater)
                    }
                    '=' => {
                        self.next();
                        self.create_token(&start, TokenKind::LessEqual)
                    }
                    _ => self.create_token(&start, TokenKind::OpenAngle),
                },
                '>' => {
                    if self.eat_opt('=') {
                        self.create_token(&start, TokenKind::GreaterEqual)
                    } else {
                        self.create_token(&start, TokenKind::CloseAngle)
                    }
                }
                '[' => self.create_token(&start, TokenKind::OpenSquare),
                ']' => self.create_token(&start, TokenKind::CloseSquare),
                '=' => {
                    if self.eat_opt('>') {
                        self.create_token(&start, TokenKind::DoubleArrow)
                    } else {
                        self.create_token(&start, TokenKind::Equal)
                    }
                }
                '!' => {
                    if self.eat('=') {
                        self.create_token(&start, TokenKind::BangEqual)
                    } else {
                        self.create_error(&start)
                    }
                }
                '+' => self.create_token(&start, TokenKind::Plus),
                '-' => {
                    if self.eat_opt('-') {
                        self.skip_to_end_of_line();
                        self.create_and_add_comment(&start, CommentKind::DelimitedComment);
                        self.lex_token()
                    } else {
                        self.create_token(&start, TokenKind::Minus)
                    }
                }
                '*' => self.create_token(&start, TokenKind::Asterisk),
                // TODO: bracketed comments
                '/' => self.create_token(&start, TokenKind::Slash),
                '%' => self.create_token(&start, TokenKind::Percent),
                '|' => {
                    if self.eat('|') {
                        self.create_token(&start, TokenKind::BarBar)
                    } else {
                        self.create_error(&start)
                    }
                }
                _ => panic!("TODO"),
            }
        }
    }
}
