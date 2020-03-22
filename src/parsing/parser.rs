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

type ElementParser<'a> = fn(&mut Parser<'a>) -> ParseTree<'a>;

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

    fn get_empty_range(&mut self) -> TextRange {
        self.position.get_empty_range()
    }

    fn eat_empty(&mut self) -> ParseTree<'a> {
        parse_tree::empty(self.get_empty_range())
    }

    fn eat_token(&mut self) -> ParseTree<'a> {
        parse_tree::token(self.advance())
    }

    fn error(&mut self, message: String) -> ParseTree<'a> {
        parse_tree::error(self.get_empty_range(), message)
    }

    fn expected_error(&mut self, expected: TokenKind) -> ParseTree<'a> {
        let message = format!("Expected {}, found {}.", expected, self.peek());
        self.error(message)
    }

    fn eat(&mut self, kind: TokenKind) -> ParseTree<'a> {
        if self.peek_kind(kind) {
            self.eat_token()
        } else {
            self.expected_error(kind)
        }
    }

    fn eat_opt(&mut self, kind: TokenKind) -> ParseTree<'a> {
        if self.peek_kind(kind) {
            self.eat_token()
        } else {
            self.eat_empty()
        }
    }

    fn parse_delimited(
        &mut self,
        start_kind: TokenKind,
        parse_element: ElementParser<'a>,
        end_kind: TokenKind,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        let start = self.eat(start_kind);
        let element = parse_element(self);
        let end = self.eat(end_kind);
        (start, element, end)
    }

    fn parse_parenthesized(
        &mut self,
        parse_element: ElementParser<'a>,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        self.parse_delimited(TokenKind::OpenParen, parse_element, TokenKind::CloseParen)
    }

    fn parse_separated_list_elements(
        &mut self,
        separator_kind: TokenKind,
        parse_element: ElementParser<'a>,
    ) -> Vec<(ParseTree<'a>, ParseTree<'a>)> {
        let mut elements = Vec::new();
        let mut seperators = Vec::new();
        elements.push(parse_element(self));
        while {
            let separator = self.eat_opt(separator_kind);
            let at_end = separator.is_empty();
            seperators.push(separator);
            !at_end
        } {}
        elements.into_iter().zip(seperators.into_iter()).collect()
    }

    // Parse non-empty separated list.
    // Terminating separator is not consumed.
    fn parse_separated_list(
        &mut self,
        separator_kind: TokenKind,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        let start_delimiter = self.eat_empty();
        let elements_and_separators =
            self.parse_separated_list_elements(separator_kind, parse_element);
        let end_delimiter = self.eat_empty();
        parse_tree::list(start_delimiter, elements_and_separators, end_delimiter)
    }

    // Parse non-empty comma separated list.
    // Terminating commas are not consumed.
    fn parse_comma_separated_list(&mut self, parse_element: ElementParser<'a>) -> ParseTree<'a> {
        self.parse_separated_list(TokenKind::Comma, parse_element)
    }

    // Parse delimited non-empty separated list.
    // Terminating separator is not permitted.
    fn parse_delimited_separated_list(
        &mut self,
        start_kind: TokenKind,
        separator_kind: TokenKind,
        parse_element: ElementParser<'a>,
        end_kind: TokenKind,
    ) -> ParseTree<'a> {
        let start_delimiter = self.eat(start_kind);
        let elements_and_separators =
            self.parse_separated_list_elements(separator_kind, parse_element);
        let end_delimiter = self.eat(end_kind);
        parse_tree::list(start_delimiter, elements_and_separators, end_delimiter)
    }

    // Parse parenthesized, non-empty comma separated list.
    // Terminating commas are not consumed.
    fn parse_parenthesized_comma_separated_list(
        &mut self,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        self.parse_delimited_separated_list(
            TokenKind::OpenParen,
            TokenKind::Comma,
            parse_element,
            TokenKind::CloseParen,
        )
    }

    // Parse optional parenthesized, non-empty comma separated list.
    // Terminating commas are not consumed.
    fn parse_parenthesized_comma_separated_list_opt(
        &mut self,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        if self.peek_kind(TokenKind::OpenParen) {
            self.parse_delimited_separated_list(
                TokenKind::OpenParen,
                TokenKind::Comma,
                parse_element,
                TokenKind::CloseParen,
            )
        } else {
            self.eat_empty()
        }
    }
}

// Presto Language specific functions
impl<'a> Parser<'a> {
    // with_? queryNoWith
    pub fn parse_query(&mut self) -> ParseTree<'a> {
        let with = self.parse_with_opt();
        // TODO
        let query_no_with = self.eat_empty();
        parse_tree::query(with, query_no_with)
    }

    // WITH RECURSIVE? namedQuery (',' namedQuery)*
    fn parse_with_opt(&mut self) -> ParseTree<'a> {
        if self.peek_kind(TokenKind::WITH) {
            let with = self.eat_token();
            let recursive = self.eat_opt(TokenKind::RECURSIVE);
            let named_queries =
                self.parse_comma_separated_list(|parser| parser.parse_named_query());
            parse_tree::with(with, recursive, named_queries)
        } else {
            self.eat_empty()
        }
    }

    // name=identifier (columnAliases)? AS '(' query ')'
    fn parse_named_query(&mut self) -> ParseTree<'a> {
        let name = self.parse_identifier();
        let column_aliases = self.parse_column_aliases_opt();
        let as_ = self.eat(TokenKind::AS);
        let (open_paren, query, close_paren) =
            self.parse_parenthesized(|parser| parser.parse_query());
        parse_tree::named_query(name, column_aliases, as_, open_paren, query, close_paren)
    }

    // identifier
    // : IDENTIFIER             #unquotedIdentifier
    // | QUOTED_IDENTIFIER      #quotedIdentifier
    // | nonReserved            #unquotedIdentifier
    // | BACKQUOTED_IDENTIFIER  #backQuotedIdentifier
    // | DIGIT_IDENTIFIER       #digitIdentifier
    // ;
    fn parse_identifier(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TokenKind::Identifier
            | TokenKind::QuotedIdentifier
            | TokenKind::BackquotedIdentifier
            | TokenKind::DigitIdentifier => self.eat_token(),
            _ => self.expected_error(TokenKind::Identifier),
        }
    }

    // columnAliases
    // : '(' identifier (',' identifier)* ')'
    fn parse_column_aliases_opt(&mut self) -> ParseTree<'a> {
        self.parse_parenthesized_comma_separated_list_opt(|parser| parser.parse_identifier())
    }
}
