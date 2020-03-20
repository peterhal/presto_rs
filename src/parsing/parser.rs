use crate::lexing::{
    lexer::Lexer, position, position::Position, predefined_names::PredefinedName,
    text_range::TextRange, token::Token, token_kind::TokenKind,
};
use crate::parsing::{parse_tree, parse_tree::ParseTree};

// The location and lexing context for a Parser.
//
// tokens contains both consumed tokens, as well as the token lookahead.
// Tokens are lexed from the lexer on demand as they are peeked for.
//
// Consuming a token advances index past the token. Lex errors in the consumed tokens
// become part of the parse results. Tokens must never be unconsumed.
struct ParsePosition<'a> {
    index: usize,
    tokens: Vec<Token<'a>>,
    lexer: Lexer<'a>,
}

impl<'a> ParsePosition<'a> {
    pub fn new(value: &'a str) -> ParsePosition<'a> {
        ParsePosition {
            index: 0,
            tokens: Vec::new(),
            lexer: Lexer::new(value),
        }
    }

    fn end_position(&self) -> Position {
        if let Some(token) = self.tokens.last() {
            token.full_end()
        } else {
            position::START
        }
    }

    fn get_token(&mut self, index: usize) -> &Token<'a> {
        while index >= self.tokens.len() {
            let new_token = self.lexer.lex_token();
            assert!(self.end_position() <= new_token.full_start());
            self.tokens.push(new_token);
        }
        &self.tokens[index]
    }

    pub fn peek_token_offset(&mut self, offset: usize) -> &Token<'a> {
        self.get_token(self.index + offset)
    }

    pub fn peek_token(&mut self) -> &Token<'a> {
        self.peek_token_offset(0)
    }

    pub fn peek_offset(&mut self, offset: usize) -> TokenKind {
        self.peek_token_offset(offset).kind
    }

    pub fn peek_kind_offset(&mut self, kind: TokenKind, offset: usize) -> bool {
        self.peek_offset(offset) == kind
    }

    pub fn peek_kind(&mut self, kind: TokenKind) -> bool {
        self.peek_kind_offset(kind, 0)
    }

    pub fn peek(&mut self) -> TokenKind {
        self.peek_offset(0)
    }

    fn get_empty_range(&mut self) -> TextRange {
        TextRange::empty(self.peek_token().full_start())
    }

    fn advance(&mut self) -> Token<'a> {
        assert!(self.index < self.tokens.len());
        // TODO: Can we avoid this clone?
        let token = self.peek_token().clone();
        self.index += 1;
        token
    }
}

pub struct Parser<'a> {
    position: ParsePosition<'a>,
}

// Language independant parser functions
impl<'a> Parser<'a> {
    pub fn new(value: &'a str) -> Parser<'a> {
        Parser {
            position: ParsePosition::new(value),
        }
    }

    fn peek_token_offset(&mut self, offset: usize) -> &Token<'a> {
        self.position.peek_token_offset(offset)
    }

    fn peek_token(&mut self) -> &Token<'a> {
        self.position.peek_token()
    }

    fn peek_offset(&mut self, offset: usize) -> TokenKind {
        self.position.peek_offset(offset)
    }

    fn peek_kind_offset(&mut self, kind: TokenKind, offset: usize) -> bool {
        self.position.peek_kind_offset(kind, offset)
    }

    fn peek_kind(&mut self, kind: TokenKind) -> bool {
        self.position.peek_kind(kind)
    }

    fn peek(&mut self) -> TokenKind {
        self.position.peek()
    }

    fn peek_predefined_name_offset(&mut self, name: PredefinedName, offset: usize) -> bool {
        let token = self.peek_token_offset(offset);
        token.kind == TokenKind::Identifier && token.value == name.to_string()
    }

    fn peek_predefined_name(&mut self, name: PredefinedName) -> bool {
        self.peek_predefined_name_offset(name, 0)
    }

    fn advance(&mut self) -> Token<'a> {
        self.position.advance()
    }

    fn eat_empty(&mut self) -> ParseTree<'a> {
        parse_tree::empty(TextRange::empty(self.peek_token().range.start))
    }

    fn eat_token(&mut self) -> ParseTree<'a> {
        parse_tree::token(self.advance())
    }

    fn eat_opt(&mut self, kind: TokenKind) -> ParseTree<'a> {
        if self.peek_kind(kind) {
            self.eat_token()
        } else {
            self.eat_empty()
        }
    }
}

// Presto Language specific functions
impl<'a> Parser<'a> {
    pub fn parse_query(&mut self) -> ParseTree<'a> {
        let with = self.parse_with_opt();
        // TODO
        let query_no_with = self.eat_empty();
        parse_tree::query(with, query_no_with)
    }

    fn parse_with_opt(&mut self) -> ParseTree<'a> {
        if self.peek_kind(TokenKind::WITH) {
            let with = self.eat_token();
            let recursive = self.eat_opt(TokenKind::RECURSIVE);
            // TODO
            let named_queries = self.eat_empty();
            parse_tree::with(with, recursive, named_queries)
        } else {
            self.eat_empty()
        }
    }
}
