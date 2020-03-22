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

    fn expected_error(&mut self, expected: &str) -> ParseTree<'a> {
        let message = format!("Expected {}, found {}.", expected, self.peek());
        self.error(message)
    }

    fn expected_error_kind(&mut self, expected: TokenKind) -> ParseTree<'a> {
        self.expected_error(expected.to_string().as_str())
    }

    fn expected_error_name(&mut self, expected: PredefinedName) -> ParseTree<'a> {
        self.expected_error(expected.to_string().as_str())
    }

    fn eat(&mut self, kind: TokenKind) -> ParseTree<'a> {
        if self.peek_kind(kind) {
            self.eat_token()
        } else {
            self.expected_error_kind(kind)
        }
    }

    fn eat_predefined_name(&mut self, name: PredefinedName) -> ParseTree<'a> {
        if self.peek_predefined_name(name) {
            self.eat_token()
        } else {
            self.expected_error_name(name)
        }
    }

    fn eat_predefined_name_opt(&mut self, name: PredefinedName) -> ParseTree<'a> {
        if self.peek_predefined_name(name) {
            self.eat_token()
        } else {
            self.eat_empty()
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
    // query
    // :  with_? queryNoWith
    pub fn parse_query(&mut self) -> ParseTree<'a> {
        let with = self.parse_with_opt();
        let query_no_with = self.parse_query_no_with();
        parse_tree::query(with, query_no_with)
    }

    // with_
    // : WITH RECURSIVE? namedQuery (',' namedQuery)*
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

    // namedQuery
    // : name=identifier (columnAliases)? AS '(' query ')'
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
            _ => self.expected_error_kind(TokenKind::Identifier),
        }
    }

    // columnAliases
    // : '(' identifier (',' identifier)* ')'
    fn parse_column_aliases_opt(&mut self) -> ParseTree<'a> {
        self.parse_parenthesized_comma_separated_list_opt(|parser| parser.parse_identifier())
    }

    // queryNoWith:
    //   queryTerm
    //   (ORDER BY sortItem (',' sortItem)*)?
    //   (LIMIT limit=(INTEGER_VALUE | ALL))?
    fn parse_query_no_with(&mut self) -> ParseTree<'a> {
        let query_term = self.parse_query_term();
        let order_by_opt = self.parse_order_by_opt();
        let limit_opt = self.parse_limit_opt();
        parse_tree::query_no_with(query_term, order_by_opt, limit_opt)
    }

    // sortItem
    // : expression ordering=(ASC | DESC)? (NULLS nullOrdering=(FIRST | LAST))?
    fn parse_sort_item(&mut self) -> ParseTree<'a> {
        let expression = self.parse_expression();
        let ordering_opt = self.parse_ordering_opt();
        let nulls = self.eat_predefined_name_opt(PredefinedName::NULLS);
        let null_ordering = if nulls.is_empty() {
            self.eat_empty()
        } else {
            self.parse_null_ordering()
        };
        parse_tree::sort_item(expression, ordering_opt, nulls, null_ordering)
    }

    fn parse_ordering_opt(&mut self) -> ParseTree<'a> {
        let asc = self.eat_predefined_name_opt(PredefinedName::ASC);
        if asc.is_empty() {
            self.eat_predefined_name_opt(PredefinedName::DESC)
        } else {
            asc
        }
    }

    fn parse_null_ordering(&mut self) -> ParseTree<'a> {
        let last = self.eat_predefined_name_opt(PredefinedName::LAST);
        if last.is_empty() {
            self.eat_predefined_name(PredefinedName::FIRST)
        } else {
            last
        }
    }

    //   (ORDER BY sortItem (',' sortItem)*)?
    fn parse_order_by_opt(&mut self) -> ParseTree<'a> {
        let order = self.eat_opt(TokenKind::ORDER);
        if order.is_empty() {
            order
        } else {
            let by = self.eat(TokenKind::BY);
            let sort_items = self.parse_comma_separated_list(|parser| parser.parse_sort_item());
            parse_tree::order_by(order, by, sort_items)
        }
    }

    //   (LIMIT limit=(INTEGER_VALUE | ALL))?
    fn parse_limit_opt(&mut self) -> ParseTree<'a> {
        let limit = self.eat_predefined_name_opt(PredefinedName::LIMIT);
        if limit.is_empty() {
            limit
        } else {
            let value = self.eat_predefined_name_opt(PredefinedName::ALL);
            let value = if value.is_empty() {
                self.eat(TokenKind::Integer)
            } else {
                value
            };
            parse_tree::limit(limit, value)
        }
    }

    // queryTerm
    // : queryPrimary                                                             #queryTermDefault
    // | left=queryTerm operator=INTERSECT setQuantifier? right=queryTerm         #setOperation
    // | left=queryTerm operator=(UNION | EXCEPT) setQuantifier? right=queryTerm  #setOperation
    fn parse_query_term(&mut self) -> ParseTree<'a> {
        // handle operator precedence here
        self.parse_union_query_term()
    }

    // | left=queryTerm operator=(UNION | EXCEPT) setQuantifier? right=queryTerm  #setOperation
    fn parse_union_query_term(&mut self) -> ParseTree<'a> {
        let mut left = self.parse_intersect_query_term();
        while {
            let op_kind = self.peek();
            op_kind == TokenKind::UNION || op_kind == TokenKind::EXCEPT
        } {
            let operator = self.eat_token();
            let set_quantifier_opt = self.parse_set_quantifier_opt();
            let right = self.parse_intersect_query_term();
            left = parse_tree::query_set_operation(left, operator, set_quantifier_opt, right);
        }
        left
    }

    // | left=queryTerm operator=INTERSECT setQuantifier? right=queryTerm         #setOperation
    fn parse_intersect_query_term(&mut self) -> ParseTree<'a> {
        let mut left = self.parse_query_primary();
        while self.peek_kind(TokenKind::INTERSECT) {
            let operator = self.eat_token();
            let set_quantifier_opt = self.parse_set_quantifier_opt();
            let right = self.parse_query_primary();
            left = parse_tree::query_set_operation(left, operator, set_quantifier_opt, right);
        }
        left
    }

    // setQuantifier
    // : DISTINCT
    // | ALL
    fn parse_set_quantifier_opt(&mut self) -> ParseTree<'a> {
        let distinct = self.eat_opt(TokenKind::DISTINCT);
        if distinct.is_empty() {
            self.eat_predefined_name_opt(PredefinedName::ALL)
        } else {
            distinct
        }
    }

    fn parse_query_primary(&mut self) -> ParseTree<'a> {
        // TODO
        self.eat_empty()
    }

    fn parse_expression(&mut self) -> ParseTree<'a> {
        // TODO
        self.eat_empty()
    }
}
