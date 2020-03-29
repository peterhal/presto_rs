use crate::lexing::{
    chars, comment::Comment, comment::CommentKind, keywords, keywords::Keyword,
    lexer_position::LexerPosition, position::Position, predefined_names::PredefinedName,
    syntax_error, syntax_error::Message, syntax_error::SyntaxError, text_range::TextRange,
    token::Token, token_kind::TokenKind,
};
use std::mem;

#[derive(Clone, Debug)]
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

    fn try_lexer(&self) -> Lexer<'a> {
        Lexer {
            input: self.input,
            position: self.position,
            errors: Vec::new(),
            comments: Vec::new(),
        }
    }

    fn mark(&self) -> LexerPosition<'a> {
        self.position
    }

    fn get_position(&self) -> Position {
        self.position.position
    }

    fn peek(&mut self) -> char {
        self.position.peek()
    }

    fn eat(&mut self, ch: char) -> bool {
        if self.eat_opt(ch) {
            true
        } else {
            let start = self.mark();
            let actual = self.peek();
            self.add_error_at(
                &start,
                syntax_error::ERROR_EXPECTED_CHAR,
                &format!("Expected '{}'; got '{}'", ch, actual),
            );
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

    fn peek_offset(&self, offset: i32) -> char {
        self.position.peek_offset(offset)
    }

    fn peek_char(&self, ch: char) -> bool {
        self.position.peek_char(ch)
    }

    fn peek_char_lower(&self, ch: char) -> bool {
        self.position.peek().eq_ignore_ascii_case(&ch)
    }

    fn peek_char_offset(&self, ch: char, offset: i32) -> bool {
        self.position.peek_char_offset(ch, offset)
    }

    pub fn at_end(&self) -> bool {
        self.position.at_end()
    }

    fn next(&mut self) -> char {
        self.position.next()
    }

    fn create_token(&mut self, start: &LexerPosition<'a>, kind: TokenKind) -> Token<'a> {
        let range = self.get_range(start);
        let value = self.get_text(start);
        let leading_comments = mem::replace(&mut self.comments, Vec::new());
        let mut errors = mem::replace(&mut self.errors, Vec::new());
        // TODO: Make consuming trailing trivia optional.
        self.lex_trailing_trivia();
        // lex_trailing_trivia may add more errors.
        errors.append(&mut self.errors);
        self.errors = Vec::new();
        let trailing_comments = mem::replace(&mut self.comments, Vec::new());
        Token {
            kind,
            range,
            value,
            errors,
            leading_comments,
            trailing_comments,
        }
    }

    fn get_range(&self, start: &LexerPosition<'a>) -> TextRange {
        start.get_range(&self.position)
    }

    fn get_text(&self, start: &LexerPosition<'a>) -> &'a str {
        start.get_text(&self.position)
    }

    fn add_error(&mut self, error: SyntaxError) {
        self.errors.push(error)
    }

    fn add_error_at(&mut self, start: &LexerPosition, error_code: i32, message: &str) {
        self.add_error(SyntaxError {
            error_code,
            messages: vec![Message {
                range: self.get_range(start),
                message: String::from(message),
            }],
        })
    }

    fn add_and_create_error(
        &mut self,
        start: &LexerPosition<'a>,
        error_code: i32,
        message: &str,
    ) -> Token<'a> {
        self.add_error_at(start, error_code, message);
        self.create_error_token(start)
    }

    fn create_error_token(&mut self, start: &LexerPosition<'a>) -> Token<'a> {
        assert!(
            !self.errors.is_empty(),
            "must add error before creating error token"
        );
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

    fn set_position(&mut self, position: &LexerPosition<'a>) {
        self.position = *position;
    }
}

// Language specific lexing goes here:
impl<'a> Lexer<'a> {
    pub fn skip_while<P>(&mut self, predicate: P) -> bool
    where
        P: Fn(char) -> bool,
    {
        self.position.skip_while(predicate)
    }

    fn skip_whitespace(&mut self) -> bool {
        self.skip_while(chars::is_whitespace)
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

    fn skip_delimited_comment_tail(&mut self, start: &LexerPosition<'a>) {
        loop {
            if self.at_end() {
                self.add_error_at(
                    start,
                    syntax_error::ERROR_UNTERMINATED_DELIMITED_COMMENT,
                    "Unterminated delimited comment.",
                );
                break;
            } else if self.peek_char('*') && self.peek_char_offset('/', 1) {
                self.next();
                self.next();
                break ();
            } else {
                self.next();
            }
        }
    }

    fn lex_trailing_trivia(&mut self) {
        assert!(self.comments.is_empty());
        assert!(self.errors.is_empty());
        let start = self.mark();
        while {
            let whitespace_start = self.mark();
            self.skip_whitespace();
            if start.line() != self.position.line() {
                self.set_position(&whitespace_start);
                false
            } else if self.peek_char('-') && self.peek_char_offset('-', 1) {
                let start_comment = self.mark();
                self.skip_to_end_of_line();
                self.create_and_add_comment(&start_comment, CommentKind::LineComment);
                false
            } else if self.peek_char('/') && self.peek_char_offset('*', 1) {
                assert!(self.errors.is_empty());
                let start_comment = self.mark();
                self.skip_delimited_comment_tail(&start_comment);
                if start.line() == self.position.line() {
                    // multiline comment, starting on the same line as the token
                    // and ends on the same line. Consume the comment.
                    self.create_and_add_comment(&start_comment, CommentKind::DelimitedComment);
                    true
                } else {
                    // multiline comment, starting on the same line as the token
                    // but ends on a different line.
                    // Keep any previous comments on this line, but rewind before this
                    // comment so that it gets attached to the next token.
                    self.set_position(&whitespace_start);
                    // Need to clear errors, which may only have occured during this
                    // delimited comment tail, which we're rewinding over.
                    self.errors = Vec::new();
                    false
                }
            } else {
                // new token on the same line as the completed token
                // discard any consumed trivia.
                self.set_position(&start);
                self.errors = Vec::new();
                self.comments = Vec::new();
                false
            }
        } {
            ()
        }
    }

    pub fn lex_string_literal(&mut self, start: &LexerPosition<'a>) -> Token<'a> {
        self.lex_any_string_literal(start, TokenKind::String)
    }

    // UNICODE_STRING
    // : 'U&\'' ( ~'\'' | '\'\'' )* '\''
    // U& already consumed
    pub fn lex_unicode_string_literal(&mut self, start: &LexerPosition<'a>) -> Token<'a> {
        self.eat('\'');
        self.lex_any_string_literal(start, TokenKind::UnicodeString)
    }

    pub fn lex_binary_literal(&mut self, start: &LexerPosition<'a>) -> Token<'a> {
        self.lex_any_string_literal(start, TokenKind::BinaryLiteral)
    }

    // leading ' already consumed
    pub fn lex_any_string_literal(
        &mut self,
        start: &LexerPosition<'a>,
        kind: TokenKind,
    ) -> Token<'a> {
        loop {
            if self.at_end() {
                return self.add_and_create_error(
                    start,
                    syntax_error::ERROR_UNTERMINATED_STRING_LITERAL,
                    "Unterminated string literal",
                );
            }
            if self.eat_opt('\'') {
                if self.peek_char('\'') {
                    self.next();
                } else {
                    return self.create_token(start, kind);
                }
            } else {
                self.next();
            }
        }
    }

    pub fn lex_quoted_identifier(&mut self, start: &LexerPosition<'a>) -> Token<'a> {
        loop {
            if self.at_end() {
                return self.add_and_create_error(
                    start,
                    syntax_error::ERROR_UNTERMINATED_QUOTED_IDENTIFIER,
                    "Unterminated quoted identifier",
                );
            }
            if self.eat_opt('"') {
                if self.peek_char('"') {
                    self.next();
                } else {
                    return self.create_token(start, TokenKind::QuotedIdentifier);
                }
            } else {
                self.next();
            }
        }
    }

    pub fn lex_back_quoted_identifier(&mut self, start: &LexerPosition<'a>) -> Token<'a> {
        loop {
            if self.at_end() {
                return self.add_and_create_error(
                    start,
                    syntax_error::ERROR_UNTERMINATED_BACK_QUOTED_IDENTIFIER,
                    "Unterminated back quoted identifier",
                );
            }
            if self.eat_opt('`') {
                if self.peek_char('`') {
                    self.next();
                } else {
                    return self.create_token(start, TokenKind::BackquotedIdentifier);
                }
            } else {
                self.next();
            }
        }
    }

    pub fn skip_digits(&mut self) {
        while chars::is_digit(self.peek()) {
            self.next();
        }
    }

    pub fn peek_fraction(&self) -> bool {
        self.peek_char('.')
    }

    pub fn peek_exponent(&self) -> bool {
        self.peek_char_lower('e') && {
            let next_char = self.peek_offset(1);
            chars::is_digit(next_char)
                || (chars::is_sign(next_char) && chars::is_digit(self.peek_offset(2)))
        }
    }

    pub fn skip_fraction(&mut self) -> bool {
        if self.peek_fraction() {
            // '.'
            self.next();
            self.skip_digits();
            true
        } else {
            false
        }
    }

    pub fn skip_exponent(&mut self) -> bool {
        if self.peek_exponent() {
            // E
            self.next();
            // sign or first digit
            self.next();
            // remaining digits
            self.skip_digits();
            true
        } else {
            false
        }
    }

    pub fn lex_number(&mut self, start: &LexerPosition<'a>, start_char: char) -> Token<'a> {
        let kind = if start_char == '.' {
            self.skip_digits();
            if self.skip_exponent() {
                TokenKind::Double
            } else {
                TokenKind::Decimal
            }
        } else {
            self.skip_digits();
            if self.peek_fraction() || self.peek_exponent() {
                self.skip_fraction();
                if self.skip_exponent() {
                    TokenKind::Double
                } else {
                    TokenKind::Decimal
                }
            } else {
                if self.skip_while(chars::is_identifier_part) {
                    TokenKind::DigitIdentifier
                } else {
                    TokenKind::Integer
                }
            }
        };
        self.create_token(start, kind)
    }

    fn peek_keyword(&mut self, keyword: Keyword) -> bool {
        self.skip_whitespace() && {
            let start = self.mark();
            self.skip_word();
            keyword.matches(self.get_text(&start))
        }
    }

    fn peek_predefined_name(&mut self, name: PredefinedName) -> bool {
        self.skip_whitespace() && {
            let start = self.mark();
            self.skip_word();
            name.matches(self.get_text(&start))
        }
    }

    fn skip_whitespace_word(&mut self) {
        self.skip_whitespace();
        self.skip_word();
    }

    fn skip_word(&mut self) {
        self.skip_while(chars::is_identifier_part);
    }

    pub fn lex_word(&mut self, start: &LexerPosition<'a>, ch: char) -> Token<'a> {
        assert!(chars::is_identifier_start(ch));
        self.skip_while(chars::is_identifier_part);
        let text = self.get_text(start);
        if let Some(keyword) = keywords::maybe_get_keyword(text) {
            self.create_token(start, keyword.to_token_kind())
        } else {
            if PredefinedName::DOUBLE.matches(text) {
                let mut lookahead = self.try_lexer();
                if lookahead.peek_predefined_name(PredefinedName::PRECISION) {
                    self.skip_whitespace_word();
                    return self.create_token(start, TokenKind::DoublePrecision);
                }
            } else if PredefinedName::TIME.matches(text) {
                let mut lookahead = self.try_lexer();
                if lookahead.peek_keyword(Keyword::WITH)
                    && lookahead.peek_predefined_name(PredefinedName::TIME)
                    && lookahead.peek_predefined_name(PredefinedName::ZONE)
                {
                    self.skip_whitespace_word();
                    self.skip_whitespace_word();
                    self.skip_whitespace_word();
                    return self.create_token(start, TokenKind::TimeWithTimeZone);
                }
            } else if PredefinedName::TIMESTAMP.matches(text) {
                let mut lookahead = self.try_lexer();
                if lookahead.peek_keyword(Keyword::WITH)
                    && lookahead.peek_predefined_name(PredefinedName::TIME)
                    && lookahead.peek_predefined_name(PredefinedName::ZONE)
                {
                    self.skip_whitespace_word();
                    self.skip_whitespace_word();
                    self.skip_whitespace_word();
                    return self.create_token(start, TokenKind::TimestampWithTimeZone);
                }
            }

            self.create_token(start, TokenKind::Identifier)
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
                '.' => {
                    if chars::is_digit(self.peek()) {
                        self.lex_number(&start, ch)
                    } else {
                        self.create_token(&start, TokenKind::Period)
                    }
                }
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
                        self.create_error_token(&start)
                    }
                }
                '+' => self.create_token(&start, TokenKind::Plus),
                '-' => {
                    if self.eat_opt('-') {
                        self.skip_to_end_of_line();
                        self.create_and_add_comment(&start, CommentKind::LineComment);
                        self.lex_token()
                    } else if self.eat_opt('>') {
                        self.create_token(&start, TokenKind::Arrow)
                    } else {
                        self.create_token(&start, TokenKind::Minus)
                    }
                }
                '*' => self.create_token(&start, TokenKind::Asterisk),
                '/' => {
                    if self.eat_opt('*') {
                        self.skip_delimited_comment_tail(&start);
                        self.create_and_add_comment(&start, CommentKind::DelimitedComment);
                        self.lex_token()
                    } else {
                        self.create_token(&start, TokenKind::Slash)
                    }
                }
                '%' => self.create_token(&start, TokenKind::Percent),
                '|' => {
                    if self.eat('|') {
                        self.create_token(&start, TokenKind::BarBar)
                    } else {
                        self.create_error_token(&start)
                    }
                }
                '\'' => self.lex_string_literal(&start),
                '"' => self.lex_quoted_identifier(&start),
                '`' => self.lex_back_quoted_identifier(&start),
                '?' => self.create_token(&start, TokenKind::Question),
                '0'..='9' => self.lex_number(&start, ch),
                // Identifier start char
                'a'..='z' | 'A'..='Z' | '_' => {
                    if ch.eq_ignore_ascii_case(&'u') && self.eat_opt('&') {
                        self.lex_unicode_string_literal(&start)
                    } else if ch.eq_ignore_ascii_case(&'x') && self.eat_opt('\'') {
                        self.lex_binary_literal(&start)
                    } else {
                        self.lex_word(&start, ch)
                    }
                }
                // TODO: multi-identifier lexemes
                _ => self.add_and_create_error(
                    &start,
                    syntax_error::ERROR_INVALID_TOKEN_START,
                    "Invalid token start character.",
                ),
            }
        }
    }
}
