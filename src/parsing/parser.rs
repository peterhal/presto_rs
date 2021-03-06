use super::{parse_tree, visit_post_order, ParseTree};
use crate::lexing::{
    predefined_names, predefined_names::PredefinedName as PN, Lexer, Token, TokenKind as TK,
};
use crate::utils::{
    position, position::Position, syntax_error, syntax_error::Message, syntax_error::SyntaxError,
    text_range, text_range::TextRange,
};
use parsing::parse_tree::ParseTree::Empty;

/// The location and lexing context for a Parser.
///
/// tokens contains both consumed tokens, as well as the token lookahead.
/// Tokens are lexed from the lexer on demand as they are peeked for.
///
/// Consuming a token advances index past the token. Lex errors in the consumed tokens
/// become part of the parse results. Tokens must never be unconsumed.
///
/// peek() methods inspect upcoming tokens without consuming input.
/// Only the advance() method consumes a token.
struct ParsePosition<'a> {
    index: usize,
    tokens: Vec<Token<'a>>,
    lexer: Lexer<'a>,
}

impl<'a> ParsePosition<'a> {
    pub fn new(value: &'a str) -> ParsePosition<'a> {
        ParsePosition {
            index: 0,
            tokens: vec![
                // kinda goofy, but we add the BOF token here so that
                // the lexer doesn't need to special case the first token
                Token::new(
                    TK::BeginningOfFile,
                    text_range::NONE,
                    &value[0..0],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            ],
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

    /// Gets a token at the index from the start of self.tokens.
    /// Will cause lexing more tokens if not enough tokens are
    /// present.
    fn get_token(&mut self, index: usize) -> &Token<'a> {
        while index >= self.tokens.len() {
            let new_token = self.lexer.lex_token();
            debug_assert!(self.end_position() <= new_token.full_start());
            self.tokens.push(new_token);
        }
        &self.tokens[index]
    }

    /// Returns the token at offset ahead of the current token in the input.
    /// Does not consume the token.
    pub fn peek_token_offset(&mut self, offset: usize) -> &Token<'a> {
        self.get_token(self.index + offset)
    }

    /// Returns the next token in the input.
    /// Does not consume the token.
    pub fn peek_token(&mut self) -> &Token<'a> {
        self.peek_token_offset(0)
    }

    /// Returns the token kind of the next token in the input.
    /// Does not consume the token.
    pub fn peek_offset(&mut self, offset: usize) -> TK {
        self.peek_token_offset(offset).kind
    }

    /// Returns the token kind of the offset token in the input.
    /// Does not consume the token.
    pub fn peek_kind_offset(&mut self, kind: TK, offset: usize) -> bool {
        self.peek_offset(offset) == kind
    }

    /// Returns true if the next token's kind matches kind.
    /// Does not consume the token.
    pub fn peek_kind(&mut self, kind: TK) -> bool {
        self.peek_kind_offset(kind, 0)
    }

    /// Returns the token kind of the next token in the input.
    /// Does not consume the token.
    pub fn peek(&mut self) -> TK {
        self.peek_offset(0)
    }

    fn get_empty_range(&mut self) -> TextRange {
        TextRange::empty(self.peek_token().full_start())
    }

    /// Consumes a token in the input.
    fn advance(&mut self) -> Token<'a> {
        debug_assert!(self.index < self.tokens.len());
        let token = self.peek_token().clone();
        self.index += 1;
        token
    }
}

/// Parser for the Presto SQL dialect.
///
/// peek() methods inspect upcoming tokens without consuming input.
/// Only the advance() method consumes a token.
///
/// eat*() methods consume a token and convert it into a Token tree.
///
/// *_opt() methods either parse a *, if present, or return an Empty tree.
///
/// parse_*() methods parse a given language syntax. They are preceded by
/// a comment indicating the grammar being parsed.
struct Parser<'a> {
    position: ParsePosition<'a>,
    errors: Vec<SyntaxError>,
}

type ElementParser<'a> = fn(&mut Parser<'a>) -> ParseTree<'a>;
type Peeker<'a> = fn(&mut Parser<'a>) -> bool;
type OffsetPeeker<'a> = fn(&mut Parser<'a>, usize) -> bool;

// Language independant parser functions
impl<'a> Parser<'a> {
    pub fn new(value: &'a str) -> Parser<'a> {
        Parser {
            position: ParsePosition::new(value),
            errors: Vec::new(),
        }
    }

    fn add_error(&mut self, error: SyntaxError) {
        self.errors.push(error);
    }

    fn add_error_of_tree(&mut self, location: &ParseTree<'a>, message: &str) {
        self.add_error(SyntaxError::from_message(
            syntax_error::ERROR_SYNTAX_ERROR,
            Message::new(location.get_range(), message.to_string()),
        ))
    }

    fn peek_token_offset(&mut self, offset: usize) -> &Token<'a> {
        self.position.peek_token_offset(offset)
    }

    fn peek_token(&mut self) -> &Token<'a> {
        self.position.peek_token()
    }

    fn peek_offset(&mut self, offset: usize) -> TK {
        self.position.peek_offset(offset)
    }

    fn peek_kind_offset(&mut self, kind: TK, offset: usize) -> bool {
        self.position.peek_kind_offset(kind, offset)
    }

    fn peek_kind(&mut self, kind: TK) -> bool {
        self.position.peek_kind(kind)
    }

    fn peek(&mut self) -> TK {
        self.position.peek()
    }

    /// If the token at offset is a predefined name, return it otherwise return None.
    fn maybe_peek_predefined_name_offset(&mut self, offset: usize) -> Option<PN> {
        let token = self.peek_token_offset(offset);
        if token.kind == TK::Identifier {
            predefined_names::maybe_get_predefined_name(token.value)
        } else {
            None
        }
    }

    /// If the next token is a predefined name, return it otherwise return None.
    fn maybe_peek_predefined_name(&mut self) -> Option<PN> {
        self.maybe_peek_predefined_name_offset(0)
    }

    /// Returns true if the token at offset is a specific predefined name.
    fn peek_predefined_name_offset(&mut self, name: PN, offset: usize) -> bool {
        self.maybe_peek_predefined_name_offset(offset) == Some(name)
    }

    /// Returns true if the next token is a specific predefined name.
    fn peek_predefined_name(&mut self, name: PN) -> bool {
        self.peek_predefined_name_offset(name, 0)
    }

    fn advance(&mut self) -> Token<'a> {
        self.position.advance()
    }

    /// Create an empty TextRange at the current position in the input.
    fn get_empty_range(&mut self) -> TextRange {
        self.position.get_empty_range()
    }

    /// Create an empty tree whose position is at the current input
    /// position.
    fn eat_empty(&mut self) -> ParseTree<'a> {
        parse_tree::empty(self.get_empty_range())
    }

    /// Consume the next token and return it wrapped in a Token tree.
    fn eat_token(&mut self) -> ParseTree<'a> {
        parse_tree::token(self.advance())
    }

    /// Create an Error tree at the current location.
    // TODO: Add error code parameter.
    fn error(&mut self, message: String) -> ParseTree<'a> {
        let result = parse_tree::error(SyntaxError::from_message(
            syntax_error::ERROR_SYNTAX_ERROR,
            Message {
                range: self.get_empty_range(),
                message,
            },
        ));
        // TODO: Remove this once we're debugged
        panic!(
            "WTF\n{}\n {}",
            self.position.lexer.input,
            format!("{:#?}", result)
        );
        // TODO: restore this once we're debugged
        // result
    }

    /// Create an Error tree at the current location with a given message.
    fn expected_error(&mut self, expected: &str) -> ParseTree<'a> {
        let message = format!("Expected {}, found {}.", expected, self.peek());
        self.error(message)
    }

    /// Creates an Error indicating that a given kind was expected.
    fn expected_error_kind(&mut self, expected: TK) -> ParseTree<'a> {
        self.expected_error(expected.to_string().as_str())
    }

    /// Creates an Error indicating that a given name was expected.
    fn expected_error_name(&mut self, expected: PN) -> ParseTree<'a> {
        self.expected_error(expected.to_string().as_str())
    }

    /// Consumes and returns the next token if its kind matches;
    /// otherwise returns an expected error token.
    fn eat(&mut self, kind: TK) -> ParseTree<'a> {
        if self.peek_kind(kind) {
            self.eat_token()
        } else {
            self.expected_error_kind(kind)
        }
    }

    /// Consumes and returns the next token if it matches the given
    /// predefined name; otherwise returns an expected error token.
    fn eat_predefined_name(&mut self, name: PN) -> ParseTree<'a> {
        if self.peek_predefined_name(name) {
            self.eat_token()
        } else {
            self.expected_error_name(name)
        }
    }

    /// Consumes and returns the next token if it matches the given
    /// predefined name; otherwise returns an empty tree.
    fn eat_predefined_name_opt(&mut self, name: PN) -> ParseTree<'a> {
        if self.peek_predefined_name(name) {
            self.eat_token()
        } else {
            self.eat_empty()
        }
    }

    /// Consumes and returns the next token if it matches the given
    /// kind; otherwise returns an empty tree.
    fn eat_opt(&mut self, kind: TK) -> ParseTree<'a> {
        if self.peek_kind(kind) {
            self.eat_token()
        } else {
            self.eat_empty()
        }
    }

    /// Parses a delimiter token, followed by a tree, followed by
    /// an end delimiter token. Returns a tuple with the 3 parsed
    /// trees.
    fn parse_delimited(
        &mut self,
        start_kind: TK,
        parse_element: ElementParser<'a>,
        end_kind: TK,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        let start = self.eat(start_kind);
        let element = parse_element(self);
        let end = self.eat(end_kind);
        (start, element, end)
    }

    /// Parses a tree enclosed in parens. Returns a tuple
    /// containing the 3 trees.
    fn parse_parenthesized(
        &mut self,
        parse_element: ElementParser<'a>,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        self.parse_delimited(TK::OpenParen, parse_element, TK::CloseParen)
    }

    /// Parse non-empty separated list elements.
    /// The second element in each pair is the separator.
    /// The separator for the last list element will always
    /// be an empty tree.
    fn parse_separated_list_elements(
        &mut self,
        separator_kind: TK,
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
        } {
            elements.push(parse_element(self));
        }
        elements.into_iter().zip(seperators.into_iter()).collect()
    }

    /// Parses the elements of a non-empty, non-separated list.
    /// The second element in each pair will always be an Empty tree.
    fn parse_list_elements(
        &mut self,
        peek_element: Peeker<'a>,
        parse_element: ElementParser<'a>,
    ) -> Vec<(ParseTree<'a>, ParseTree<'a>)> {
        let mut elements = Vec::new();
        while elements.len() == 0 || peek_element(self) {
            elements.push((parse_element(self), self.eat_empty()));
        }
        elements
    }

    /// Parse possibly empty separated list.
    fn parse_separated_list_elements_opt(
        &mut self,
        separator_kind: TK,
        peek_element: Peeker<'a>,
        parse_element: ElementParser<'a>,
    ) -> Vec<(ParseTree<'a>, ParseTree<'a>)> {
        if peek_element(self) {
            self.parse_separated_list_elements(separator_kind, parse_element)
        } else {
            Vec::new()
        }
    }

    /// Parse non-empty list.
    fn parse_list(
        &mut self,
        peek_element: Peeker<'a>,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        let start_delimiter = self.eat_empty();
        let elements_and_separators = self.parse_list_elements(peek_element, parse_element);
        let end_delimiter = self.eat_empty();
        parse_tree::list(start_delimiter, elements_and_separators, end_delimiter)
    }

    /// Parse non-empty separated list.
    /// Terminating separator is not consumed.
    fn parse_separated_list(
        &mut self,
        separator_kind: TK,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        let start_delimiter = self.eat_empty();
        let elements_and_separators =
            self.parse_separated_list_elements(separator_kind, parse_element);
        let end_delimiter = self.eat_empty();
        parse_tree::list(start_delimiter, elements_and_separators, end_delimiter)
    }

    /// Parse possibly-empty separated list.
    /// Terminating separator is not consumed.
    fn parse_separated_list_opt(
        &mut self,
        separator_kind: TK,
        peek_element: Peeker<'a>,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        let start_delimiter = self.eat_empty();
        let elements_and_separators =
            self.parse_separated_list_elements_opt(separator_kind, peek_element, parse_element);
        let end_delimiter = self.eat_empty();
        parse_tree::list(start_delimiter, elements_and_separators, end_delimiter)
    }

    /// Parse non-empty comma separated list.
    /// Terminating commas are not consumed.
    fn parse_comma_separated_list(&mut self, parse_element: ElementParser<'a>) -> ParseTree<'a> {
        self.parse_separated_list(TK::Comma, parse_element)
    }

    /// Parse possibly-empty comma separated list.
    /// Terminating commas are not consumed.
    fn parse_comma_separated_list_opt(
        &mut self,
        peek_element: Peeker<'a>,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        self.parse_separated_list_opt(TK::Comma, peek_element, parse_element)
    }

    /// Parse delimited non-empty separated list.
    /// Terminating separator is not permitted.
    fn parse_delimited_separated_list(
        &mut self,
        start_kind: TK,
        separator_kind: TK,
        parse_element: ElementParser<'a>,
        end_kind: TK,
    ) -> ParseTree<'a> {
        let start_delimiter = self.eat(start_kind);
        let elements_and_separators =
            self.parse_separated_list_elements(separator_kind, parse_element);
        let end_delimiter = self.eat(end_kind);
        parse_tree::list(start_delimiter, elements_and_separators, end_delimiter)
    }

    /// Parse delimited possibly-empty separated list.
    /// Terminating separator is not permitted.
    fn parse_delimited_separated_list_opt(
        &mut self,
        start_kind: TK,
        separator_kind: TK,
        peek_element: Peeker<'a>,
        parse_element: ElementParser<'a>,
        end_kind: TK,
    ) -> ParseTree<'a> {
        let start_delimiter = self.eat(start_kind);
        let elements_and_separators =
            self.parse_separated_list_elements_opt(separator_kind, peek_element, parse_element);
        let end_delimiter = self.eat(end_kind);
        parse_tree::list(start_delimiter, elements_and_separators, end_delimiter)
    }

    /// Parse parenthesized, non-empty comma separated list.
    /// Terminating commas are not consumed.
    fn parse_parenthesized_comma_separated_list(
        &mut self,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        self.parse_delimited_separated_list(TK::OpenParen, TK::Comma, parse_element, TK::CloseParen)
    }

    /// Parse optional parenthesized, non-empty comma separated list.
    /// Terminating commas are not consumed.
    fn parse_parenthesized_comma_separated_list_opt(
        &mut self,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        if self.peek_kind(TK::OpenParen) {
            self.parse_delimited_separated_list(
                TK::OpenParen,
                TK::Comma,
                parse_element,
                TK::CloseParen,
            )
        } else {
            self.eat_empty()
        }
    }

    /// Parse parenthesized, possibly empty comma separated list.
    /// Terminating commas are not consumed.
    fn parse_parenthesized_comma_separated_opt_list(
        &mut self,
        peek_element: Peeker<'a>,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        self.parse_delimited_separated_list_opt(
            TK::OpenParen,
            TK::Comma,
            peek_element,
            parse_element,
            TK::CloseParen,
        )
    }

    /// Parses a grammar entrypoint; ensures all input is consumed.
    fn parse_entrypoint(&mut self, parse_element: ElementParser<'a>) -> ParseTree<'a> {
        let (bof, tree, eof) =
            self.parse_delimited(TK::BeginningOfFile, parse_element, TK::EndOfFile);
        parse_tree::entrypoint(bof, tree, eof)
    }
}

// Presto Language specific functions
//
// Methods here are prefixed with a comment indicating the grammar
// production being parsed.
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
        if self.peek_kind(TK::WITH) {
            let with = self.eat_token();
            let recursive = self.eat_opt(TK::RECURSIVE);
            let named_queries =
                self.parse_comma_separated_list(|parser| parser.parse_named_query());
            parse_tree::with(with, recursive, named_queries)
        } else {
            self.eat_empty()
        }
    }

    fn parse_parenthesized_query(&mut self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        self.parse_parenthesized(|parser| parser.parse_query())
    }

    // namedQuery
    // : name=identifier (columnAliases)? AS '(' query ')'
    fn parse_named_query(&mut self) -> ParseTree<'a> {
        let name = self.parse_identifier();
        let column_aliases = self.parse_column_aliases_opt();
        let as_ = self.eat(TK::AS);
        let (open_paren, query, close_paren) = self.parse_parenthesized_query();
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
        if self.peek_identifier() {
            self.eat_token()
        } else {
            self.expected_error_kind(TK::Identifier)
        }
    }

    fn peek_identifier_offset(&mut self, offset: usize) -> bool {
        match self.peek_offset(offset) {
            TK::Identifier
            | TK::QuotedIdentifier
            | TK::BackquotedIdentifier
            | TK::DigitIdentifier => true,
            _ => false,
        }
    }

    fn peek_identifier(&mut self) -> bool {
        self.peek_identifier_offset(0)
    }

    fn parse_identifier_opt(&mut self) -> ParseTree<'a> {
        if self.peek_identifier() {
            self.eat_token()
        } else {
            self.eat_empty()
        }
    }

    // columnAliases
    // : '(' identifier (',' identifier)* ')'
    fn parse_column_aliases_opt(&mut self) -> ParseTree<'a> {
        if self.peek_column_aliases() {
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_identifier())
        } else {
            self.eat_empty()
        }
    }

    fn peek_column_aliases(&mut self) -> bool {
        // need to disambiguate with query in an insert into
        self.peek_kind(TK::OpenParen) && self.peek_identifier_offset(1)
    }

    // queryNoWith:
    //   queryTerm
    //   (ORDER BY sortItem (',' sortItem)*)?
    //   (LIMIT limit=(INTEGER_VALUE | ALL))?
    fn parse_query_no_with(&mut self) -> ParseTree<'a> {
        let query_term = self.parse_query_term();
        self.parse_query_no_with_tail(query_term)
    }

    fn peek_query_no_with_tail(&mut self) -> bool {
        self.peek_kind(TK::ORDER) || self.peek_limit_offset(0)
    }

    fn parse_query_no_with_tail(&mut self, query_term: ParseTree<'a>) -> ParseTree<'a> {
        let order_by_opt = self.parse_order_by_opt();
        let limit_opt = self.parse_limit_opt();
        parse_tree::query_no_with(query_term, order_by_opt, limit_opt)
    }

    // sortItem
    // : expression ordering=(ASC | DESC)? (NULLS nullOrdering=(FIRST | LAST))?
    fn parse_sort_item(&mut self) -> ParseTree<'a> {
        let expression = self.parse_expression();
        let ordering_opt = self.parse_ordering_opt();
        let nulls = self.eat_predefined_name_opt(PN::NULLS);
        let null_ordering = if nulls.is_empty() {
            self.eat_empty()
        } else {
            self.parse_null_ordering()
        };
        parse_tree::sort_item(expression, ordering_opt, nulls, null_ordering)
    }

    fn parse_ordering_opt(&mut self) -> ParseTree<'a> {
        let asc = self.eat_predefined_name_opt(PN::ASC);
        if asc.is_empty() {
            self.eat_predefined_name_opt(PN::DESC)
        } else {
            asc
        }
    }

    fn parse_null_ordering(&mut self) -> ParseTree<'a> {
        let last = self.eat_predefined_name_opt(PN::LAST);
        if last.is_empty() {
            self.eat_predefined_name(PN::FIRST)
        } else {
            last
        }
    }

    //   (ORDER BY sortItem (',' sortItem)*)?
    fn parse_order_by_opt(&mut self) -> ParseTree<'a> {
        let order = self.eat_opt(TK::ORDER);
        if order.is_empty() {
            order
        } else {
            let by = self.eat(TK::BY);
            let sort_items = self.parse_comma_separated_list(|parser| parser.parse_sort_item());
            parse_tree::order_by(order, by, sort_items)
        }
    }

    //   (LIMIT limit=(INTEGER_VALUE | ALL))?
    fn parse_limit_opt(&mut self) -> ParseTree<'a> {
        let limit = self.eat_predefined_name_opt(PN::LIMIT);
        if limit.is_empty() {
            limit
        } else {
            let value = self.eat_predefined_name_opt(PN::ALL);
            let value = if value.is_empty() {
                self.eat(TK::Integer)
            } else {
                value
            };
            parse_tree::limit(limit, value)
        }
    }

    fn peek_limit_offset(&mut self, offset: usize) -> bool {
        self.peek_predefined_name_offset(PN::LIMIT, offset)
            && (self.peek_predefined_name_offset(PN::ALL, offset + 1)
                || self.peek_kind_offset(TK::Integer, offset + 1))
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
        let left = self.parse_intersect_query_term();
        self.parse_union_query_term_tail(left)
    }

    fn parse_union_query_term_tail(&mut self, left: ParseTree<'a>) -> ParseTree<'a> {
        let mut left = left;
        while self.peek_union_query_term_tail() {
            let operator = self.eat_token();
            let set_quantifier_opt = self.parse_set_quantifier_opt(|_parser, _offset| true);
            let right = self.parse_intersect_query_term();
            left = parse_tree::query_set_operation(left, operator, set_quantifier_opt, right);
        }
        left
    }

    fn peek_union_query_term_tail(&mut self) -> bool {
        let op_kind = self.peek();
        op_kind == TK::UNION || op_kind == TK::EXCEPT
    }

    // | left=queryTerm operator=INTERSECT setQuantifier? right=queryTerm         #setOperation
    fn parse_intersect_query_term(&mut self) -> ParseTree<'a> {
        let left = self.parse_query_primary();
        self.parse_intersect_query_term_tail(left)
    }

    fn parse_intersect_query_term_tail(&mut self, left: ParseTree<'a>) -> ParseTree<'a> {
        let mut left = left;
        while self.peek_intersect_query_term_tail() {
            let operator = self.eat_token();
            let set_quantifier_opt = self.parse_set_quantifier_opt(|_parser, _offset| true);
            let right = self.parse_query_primary();
            left = parse_tree::query_set_operation(left, operator, set_quantifier_opt, right);
        }
        left
    }

    fn parse_query_primary_tail(&mut self, query_primary: ParseTree<'a>) -> ParseTree<'a> {
        let query_intersect = self.parse_intersect_query_term_tail(query_primary);
        let query_term = self.parse_union_query_term_tail(query_intersect);
        self.parse_query_no_with_tail(query_term)
    }

    fn peek_intersect_query_term_tail(&mut self) -> bool {
        self.peek_kind(TK::INTERSECT)
    }

    // setQuantifier
    // : DISTINCT
    // | ALL
    fn parse_set_quantifier_opt(&mut self, peek_tail: OffsetPeeker<'a>) -> ParseTree<'a> {
        let distinct = self.eat_opt(TK::DISTINCT);
        if distinct.is_empty() && self.peek_predefined_name(PN::ALL) && peek_tail(self, 1) {
            self.eat_token()
        } else {
            distinct
        }
    }

    fn peek_query_primary_offset(&mut self, offset: usize) -> bool {
        match self.peek_offset(offset) {
            TK::SELECT | TK::TABLE | TK::VALUES => true,
            TK::OpenParen => self.peek_query_primary_offset(offset + 1),
            _ => false,
        }
    }

    fn peek_query_offset(&mut self, offset: usize) -> bool {
        self.peek_kind_offset(TK::WITH, offset) || self.peek_query_primary_offset(offset)
    }

    // queryPrimary
    // : querySpecification                   #queryPrimaryDefault
    // | TABLE qualifiedName                  #table
    // | VALUES expression (',' expression)*  #inlineTable
    // | '(' queryNoWith  ')'                 #subquery
    fn parse_query_primary(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TK::SELECT => self.parse_query_specification(),
            TK::TABLE => self.parse_table(),
            TK::VALUES => self.parse_inline_table(),
            TK::OpenParen => self.parse_subquery(),
            _ => self.eat(TK::SELECT),
        }
    }

    // | '(' queryNoWith  ')'                 #subquery
    fn parse_subquery(&mut self) -> ParseTree<'a> {
        let (open_paren, query_no_with, close_paren) =
            self.parse_parenthesized(|parser| parser.parse_query_no_with());
        parse_tree::subquery(open_paren, query_no_with, close_paren)
    }

    // | VALUES expression (',' expression)*  #inlineTable
    fn parse_inline_table(&mut self) -> ParseTree<'a> {
        let values = self.eat(TK::VALUES);
        let expressions = self.parse_comma_separated_list(|parser| parser.parse_expression());
        parse_tree::inline_table(values, expressions)
    }

    // | TABLE qualifiedName                  #table
    fn parse_table(&mut self) -> ParseTree<'a> {
        let table = self.eat(TK::TABLE);
        let qualified_name = self.parse_qualified_name();
        parse_tree::table(table, qualified_name)
    }

    // querySpecification
    // : SELECT setQuantifier? selectItem (',' selectItem)*
    //   (FROM relation (',' relation)*)?
    //   (WHERE where=booleanExpression)?
    //   (GROUP BY groupBy)?
    //   (HAVING having=booleanExpression)?
    fn parse_query_specification(&mut self) -> ParseTree<'a> {
        let select = self.eat(TK::SELECT);
        let set_quantifier_opt =
            self.parse_set_quantifier_opt(|parser, offset| parser.peek_select_item_offset(offset));
        let select_items = self.parse_comma_separated_list(|parser| parser.parse_select_item());
        let from = self.eat_opt(TK::FROM);
        let relations = if from.is_empty() {
            self.eat_empty()
        } else {
            self.parse_comma_separated_list(|parser| parser.parse_relation())
        };
        let where_ = self.eat_opt(TK::WHERE);
        let where_predicate = if where_.is_empty() {
            self.eat_empty()
        } else {
            self.parse_boolean_expression()
        };
        let group = self.eat_opt(TK::GROUP);
        let (by, group_by) = if group.is_empty() {
            (self.eat_empty(), self.eat_empty())
        } else {
            let by = self.eat(TK::BY);
            let group_by = self.parse_group_by();
            (by, group_by)
        };
        let having = self.eat_opt(TK::HAVING);
        let having_predicate = if having.is_empty() {
            self.eat_empty()
        } else {
            self.parse_boolean_expression()
        };
        parse_tree::query_specification(
            select,
            set_quantifier_opt,
            select_items,
            from,
            relations,
            where_,
            where_predicate,
            group,
            by,
            group_by,
            having,
            having_predicate,
        )
    }

    // selectItem
    // : expression (AS? identifier)?  #selectSingle
    // | qualifiedName '.' ASTERISK    #selectAll
    // | ASTERISK                      #selectAll
    fn parse_select_item(&mut self) -> ParseTree<'a> {
        let asterisk = self.eat_opt(TK::Asterisk);
        if asterisk.is_empty() {
            if self.peek_qualified_select_all() {
                let qualifier = self.parse_qualified_name();
                let period = self.eat(TK::Period);
                let asterisk = self.eat(TK::Asterisk);
                parse_tree::qualified_select_all(qualifier, period, asterisk)
            } else {
                let expression = self.parse_expression();
                let as_ = self.eat_opt(TK::AS);
                let identifier = if as_.is_empty() {
                    self.parse_identifier_opt()
                } else {
                    self.parse_identifier()
                };
                parse_tree::select_item(expression, as_, identifier)
            }
        } else {
            parse_tree::select_all(asterisk)
        }
    }

    fn peek_select_item_offset(&mut self, offset: usize) -> bool {
        self.peek_expression_offset(offset) || self.peek_kind_offset(TK::Asterisk, offset)
    }

    fn peek_qualified_select_all(&mut self) -> bool {
        let mut offset = 0;
        while self.peek_identifier_offset(offset) {
            offset += 1;
            if self.peek_kind_offset(TK::Period, offset) {
                offset += 1;
            } else {
                return false;
            }
        }
        offset > 0 && self.peek_kind_offset(TK::Asterisk, offset)
    }

    // relation
    // : left=relation
    //   ( CROSS JOIN right=sampledRelation
    //   | joinType JOIN rightRelation=relation joinCriteria
    //   | NATURAL joinType JOIN right=sampledRelation
    //   )                                           #joinRelation
    // | sampledRelation                             #relationDefault
    fn parse_relation(&mut self) -> ParseTree<'a> {
        let left = self.parse_sampled_relation();
        self.parse_join_relation_tail(left)
    }

    fn peek_join_relation_tail(&mut self) -> bool {
        match self.peek() {
            TK::CROSS | TK::JOIN | TK::INNER | TK::LEFT | TK::RIGHT | TK::FULL | TK::NATURAL => {
                true
            }
            _ => false,
        }
    }

    fn parse_join_relation_tail(&mut self, left: ParseTree<'a>) -> ParseTree<'a> {
        let mut left = left;
        while self.peek_join_relation_tail() {
            left = match self.peek() {
                TK::CROSS => {
                    let cross = self.eat(TK::CROSS);
                    let join = self.eat(TK::JOIN);
                    let right = self.parse_sampled_relation();
                    parse_tree::cross_join(left, cross, join, right)
                }
                TK::JOIN | TK::INNER | TK::LEFT | TK::RIGHT | TK::FULL => {
                    let join_type = self.parse_join_type();
                    let join = self.eat(TK::JOIN);
                    let right = self.parse_relation();
                    let join_criteria = self.parse_join_criteria();
                    parse_tree::join(left, join_type, join, right, join_criteria)
                }
                TK::NATURAL => {
                    let natural = self.eat(TK::CROSS);
                    let join_type = self.parse_join_type();
                    let join = self.eat(TK::JOIN);
                    let right = self.parse_sampled_relation();
                    parse_tree::natural_join(left, natural, join_type, join, right)
                }
                _ => panic!("Unexpected join tail"),
            }
        }
        left
    }

    // joinType
    // : INNER?
    // | LEFT OUTER?
    // | RIGHT OUTER?
    // | FULL OUTER?
    fn parse_join_type(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TK::INNER => self.eat(TK::INNER),
            TK::LEFT | TK::RIGHT | TK::FULL => {
                let kind = self.eat_token();
                let outer_opt = self.eat_opt(TK::OUTER);
                parse_tree::outer_join_kind(kind, outer_opt)
            }
            _ => self.eat_empty(),
        }
    }

    // joinCriteria
    // : ON booleanExpression
    // | USING '(' identifier (',' identifier)* ')'
    fn parse_join_criteria(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TK::ON => {
                let on = self.eat(TK::ON);
                let predicate = self.parse_boolean_expression();
                parse_tree::on_join_criteria(on, predicate)
            }
            TK::USING => {
                let using = self.eat(TK::USING);
                let names = self
                    .parse_parenthesized_comma_separated_list(|parser| parser.parse_identifier());
                parse_tree::using_join_criteria(using, names)
            }
            _ => self.expected_error("join criteria"),
        }
    }

    // sampledRelation
    // : aliasedRelation (
    //     TABLESAMPLE sampleType '(' percentage=expression ')'
    //   )?
    fn parse_sampled_relation(&mut self) -> ParseTree<'a> {
        let relation_primary = self.parse_relation_primary();
        self.parse_sampled_relation_tail(relation_primary)
    }

    fn parse_sampled_relation_tail(&mut self, relation_primary: ParseTree<'a>) -> ParseTree<'a> {
        let aliased_relation = self.parse_aliased_relation_tail(relation_primary);
        let tablesample = self.eat_predefined_name_opt(PN::TABLESAMPLE);
        if tablesample.is_empty() {
            aliased_relation
        } else {
            let sample_type = self.parse_sample_type();
            let (open_paren, expression, close_paren) =
                self.parse_parenthesized(|parser| parser.parse_expression());
            parse_tree::sampled_relation(
                aliased_relation,
                tablesample,
                sample_type,
                open_paren,
                expression,
                close_paren,
            )
        }
    }

    fn peek_sampled_relation_tail(&mut self) -> bool {
        self.peek_aliased_relation_tail_offset(0) || self.peek_tablesample_suffix_offset(0)
    }

    fn peek_tablesample_suffix_offset(&mut self, offset: usize) -> bool {
        self.peek_predefined_name_offset(PN::TABLESAMPLE, offset)
            && self.peek_sample_type_offset(offset + 1)
    }

    fn peek_sample_type_offset(&mut self, offset: usize) -> bool {
        match self.maybe_peek_predefined_name_offset(offset) {
            Some(PN::BERNOULLI) | Some(PN::SYSTEM) => true,
            _ => false,
        }
    }

    // sampleType
    // : BERNOULLI
    // | SYSTEM
    fn parse_sample_type(&mut self) -> ParseTree<'a> {
        if self.peek_sample_type_offset(0) {
            self.eat_token()
        } else {
            self.expected_error("sample type")
        }
    }

    // aliasedRelation
    // : relationPrimary (AS? identifier columnAliases?)?
    fn peek_aliased_relation_tail_offset(&mut self, offset: usize) -> bool {
        (self.peek_kind_offset(TK::AS, offset) || self.peek_identifier_offset(offset)) &&
        // need to avoid consuming a TABLESAMPLE as an alias
        // This is due to the ANTLR grammar being recursive
        // through the relation production.
        !self.peek_tablesample_suffix_offset(offset)
        // need to avoid consuming the LIMIT(non-keyword) as an alias.
        && !self.peek_limit_offset(offset)
    }

    fn parse_aliased_relation_tail(&mut self, relation_primary: ParseTree<'a>) -> ParseTree<'a> {
        if self.peek_aliased_relation_tail_offset(0) {
            let as_opt = self.eat_opt(TK::AS);
            let identifier = self.parse_identifier();
            let column_aliases_opt = self.parse_column_aliases_opt();
            parse_tree::aliased_relation(relation_primary, as_opt, identifier, column_aliases_opt)
        } else {
            relation_primary
        }
    }

    fn peek_query_primary_follow(&mut self) -> bool {
        self.peek_intersect_query_term_tail()
            || self.peek_union_query_term_tail()
            || self.peek_query_no_with_tail()
    }

    // yields one of:
    //   one of several relation trees:
    //          joins, sampled, subquery_relation
    //   query - possibly with or without a with clause
    //   query_no_with
    //   relation_or_query
    fn parse_relation_or_query(&mut self) -> ParseTree<'a> {
        if self.peek_kind(TK::OpenParen) {
            let (open_paren, relation_or_query, close_paren) =
                self.parse_parenthesized(|parser| parser.parse_relation_or_query());
            let must_be_subquery_relation =
                relation_or_query.is_query() && !relation_or_query.as_query().with.is_empty();
            if must_be_subquery_relation {
                let subquery_relation =
                    parse_tree::subquery_relation(open_paren, relation_or_query, close_paren);
                let sampled_relation = self.parse_sampled_relation_tail(subquery_relation);
                self.parse_join_relation_tail(sampled_relation)
            } else if {
                let can_be_query_primary = !must_be_subquery_relation
                    && (relation_or_query.is_query_no_with()
                        || relation_or_query.is_query()
                        || relation_or_query.is_relation_or_query());
                can_be_query_primary
            } {
                // it is possible for both relation_tail and query_primary_tail
                // to be true: when followed by LIMIT x; which can only be a query.
                let relation_tail =
                    self.peek_join_relation_tail() || self.peek_sampled_relation_tail();
                let must_be_query_tail = self.peek_query_primary_follow();
                if relation_tail && !must_be_query_tail {
                    let subquery_relation =
                        parse_tree::subquery_relation(open_paren, relation_or_query, close_paren);
                    let sampled_relation = self.parse_sampled_relation_tail(subquery_relation);
                    self.parse_join_relation_tail(sampled_relation)
                } else if must_be_query_tail {
                    let subquery = parse_tree::subquery(open_paren, relation_or_query, close_paren);
                    // this yields a query_no_with
                    self.parse_query_primary_tail(subquery)
                } else {
                    // we have a query which can be consumed as either
                    // a subquery or a subquery_relation...
                    // make the decision up the tree.
                    parse_tree::relation_or_query(open_paren, relation_or_query, close_paren)
                }
            } else {
                let sampled_relation = self.parse_sampled_relation_tail(
                    parse_tree::parenthesized_relation(open_paren, relation_or_query, close_paren),
                );
                self.parse_join_relation_tail(sampled_relation)
            }
        } else if self.peek_query_offset(0) {
            // yields a query
            self.parse_query()
        } else {
            self.parse_relation()
        }
    }

    // relationPrimary
    // : qualifiedName                                                   #tableName
    // | '(' query ')'                                                   #subqueryRelation
    // | UNNEST '(' expression (',' expression)* ')' (WITH ORDINALITY)?  #unnest
    // | LATERAL '(' query ')'                                           #lateral
    // | '(' relation ')'                                                #parenthesizedRelation
    fn parse_relation_primary(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TK::OpenParen => {
                let (open_paren, relation_or_query, close_paren) =
                    self.parse_parenthesized(|parser| parser.parse_relation_or_query());
                if relation_or_query.is_query()
                    || relation_or_query.is_query_no_with()
                    || relation_or_query.is_relation_or_query()
                {
                    parse_tree::subquery_relation(open_paren, relation_or_query, close_paren)
                } else {
                    parse_tree::parenthesized_relation(open_paren, relation_or_query, close_paren)
                }
            }
            TK::UNNEST => self.parse_unnest(),
            _ => {
                if self.peek_predefined_name(PN::LATERAL) && self.peek_kind_offset(TK::OpenParen, 1)
                {
                    self.parse_lateral()
                } else {
                    self.parse_table_name()
                }
            }
        }
    }

    // : qualifiedName                                                   #tableName
    fn parse_table_name(&mut self) -> ParseTree<'a> {
        let name = self.parse_qualified_name();
        parse_tree::table_name(name)
    }

    // | LATERAL '(' query ')'                                           #lateral
    fn parse_lateral(&mut self) -> ParseTree<'a> {
        let lateral = self.eat_predefined_name(PN::LATERAL);
        let (open_paren, query, close_paren) = self.parse_parenthesized_query();
        parse_tree::lateral(lateral, open_paren, query, close_paren)
    }

    // | UNNEST '(' expression (',' expression)* ')' (WITH ORDINALITY)?  #unnest
    fn parse_unnest(&mut self) -> ParseTree<'a> {
        let unnest = self.eat(TK::UNNEST);
        let expressions =
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_expression());
        let with = self.eat_opt(TK::WITH);
        let ordinality = if with.is_empty() {
            self.eat_empty()
        } else {
            self.eat_predefined_name(PN::ORDINALITY)
        };
        parse_tree::unnest(unnest, expressions, with, ordinality)
    }

    // groupBy
    // : setQuantifier? groupingElement (',' groupingElement)*
    fn parse_group_by(&mut self) -> ParseTree<'a> {
        let set_quantifier_opt = self
            .parse_set_quantifier_opt(|parser, offset| parser.peek_grouping_element_offset(offset));
        let grouping_elements =
            self.parse_comma_separated_list(|parser| parser.parse_grouping_element());
        parse_tree::group_by(set_quantifier_opt, grouping_elements)
    }

    // groupingElement
    // : groupingSet                                            #singleGroupingSet
    // | ROLLUP '(' (expression (',' expression)*)? ')'         #rollup
    // | CUBE '(' (expression (',' expression)*)? ')'           #cube
    // | GROUPING SETS '(' groupingSet (',' groupingSet)* ')'   #multipleGroupingSets
    fn parse_grouping_element(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TK::ROLLUP => self.parse_rollup(),
            TK::CUBE => self.parse_cube(),
            TK::GROUPING => self.parse_grouping_sets(),
            _ => self.parse_grouping_set(),
        }
    }

    fn peek_grouping_element_offset(&mut self, offset: usize) -> bool {
        match self.peek_offset(offset) {
            TK::ROLLUP | TK::CUBE | TK::GROUPING => true,
            _ => self.peek_expression_offset(offset),
        }
    }

    // | ROLLUP '(' (expression (',' expression)*)? ')'         #rollup
    fn parse_rollup(&mut self) -> ParseTree<'a> {
        let rollup = self.eat(TK::ROLLUP);
        let expressions =
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_expression());
        parse_tree::rollup(rollup, expressions)
    }

    // | CUBE '(' (expression (',' expression)*)? ')'           #cube
    fn parse_cube(&mut self) -> ParseTree<'a> {
        let cube = self.eat(TK::CUBE);
        let expressions =
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_expression());
        parse_tree::cube(cube, expressions)
    }

    // | GROUPING SETS '(' groupingSet (',' groupingSet)* ')'   #multipleGroupingSets
    fn parse_grouping_sets(&mut self) -> ParseTree<'a> {
        let grouping = self.eat(TK::GROUPING);
        let sets = self.eat_predefined_name(PN::SETS);
        let grouping_sets =
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_grouping_set());
        parse_tree::grouping_sets(grouping, sets, grouping_sets)
    }

    // groupingSet
    // : '(' (expression (',' expression)*)? ')'
    // | expression
    fn parse_grouping_set(&mut self) -> ParseTree<'a> {
        // This is a subset of expression, except that it permits '()'
        // parenthesized expressions will show up as
        // either a row constructor or a paren expression.
        let elements = if self.peek_kind(TK::OpenParen) && self.peek_kind_offset(TK::CloseParen, 1)
        {
            parse_tree::empty_grouping_set(self.eat(TK::OpenParen), self.eat(TK::CloseParen))
        } else {
            self.parse_expression()
        };
        parse_tree::grouping_set(elements)
    }

    // expression
    // : booleanExpression
    fn parse_expression(&mut self) -> ParseTree<'a> {
        self.parse_boolean_expression()
    }

    fn peek_expression(&mut self) -> bool {
        self.peek_expression_offset(0)
    }

    fn peek_expression_offset(&mut self, offset: usize) -> bool {
        match self.peek_offset(offset) {
            TK::NOT
            | TK::Plus
            | TK::Minus

            // : NULL                                                                                #nullLiteral
            | TK::NULL
            // | DOUBLE_PRECISION string                                                             #typeConstructor
            | TK::DoublePrecision
            // | booleanValue                                                                        #booleanLiteral
            | TK::TRUE | TK::FALSE
            // | number                                                                              #numericLiteral
            | TK::Decimal | TK::Double | TK::Integer
            // | string                                                                              #stringLiteral
            | TK::String | TK::UnicodeString
            // | BINARY_LITERAL                                                                      #binaryLiteral
            | TK::BinaryLiteral
            // | '?'                                                                                 #parameter
            | TK::Question
            // // This is an extension to ANSI SQL, which considers EXISTS to be a <boolean expression>
            // | EXISTS '(' query ')'                                                                #exists
            | TK::EXISTS
            // | CASE valueExpression whenClause+ (ELSE elseExpression=expression)? END              #simpleCase
            // | CASE whenClause+ (ELSE elseExpression=expression)? END                              #searchedCase
            | TK::CASE
            // | CAST '(' expression AS type_ ')'                                                     #cast
            | TK::CAST
            // | name=CURRENT_DATE                                                                   #specialDateTimeFunction
            | TK::CURRENT_DATE
            // | name=CURRENT_TIME ('(' precision=INTEGER_VALUE ')')?                                #specialDateTimeFunction
            | TK::CURRENT_TIME
            // | name=CURRENT_TIMESTAMP ('(' precision=INTEGER_VALUE ')')?                           #specialDateTimeFunction
            | TK::CURRENT_TIMESTAMP
            // | name=LOCALTIME ('(' precision=INTEGER_VALUE ')')?                                   #specialDateTimeFunction
            | TK::LOCALTIME
            // | name=LOCALTIMESTAMP ('(' precision=INTEGER_VALUE ')')?                              #specialDateTimeFunction
            | TK::LOCALTIMESTAMP
            // | name=CURRENT_USER                                                                   #currentUser
            | TK::CURRENT_USER
            // | name=CURRENT_PATH                                                                   #currentPath
            | TK::CURRENT_PATH
            // | NORMALIZE '(' valueExpression (',' normalForm)? ')'                                 #normalize
            | TK::NORMALIZE
            // | EXTRACT '(' identifier FROM valueExpression ')'                                     #extract
            | TK::EXTRACT
            // | GROUPING '(' (qualifiedName (',' qualifiedName)*)? ')'                              #groupingOperation
            | TK::GROUPING
            // | configureExpression                                                                 #conf
            | TK::CONFIGURE

            // | '(' expression (',' expression)+ ')'                                                #rowConstructor
            // | '(' (identifier (',' identifier)*)? ')' '->' expression                             #lambda
            // | '(' query ')'                                                                       #subqueryExpression
            // | '(' expression ')'                                                                  #parenthesizedExpression
            | TK::OpenParen

            // | interval                                                                            #intervalLiteral
            // | identifier string                                                                   #typeConstructor
            // | POSITION '(' valueExpression IN valueExpression ')'                                 #position
            // | ROW '(' expression (',' expression)* ')'                                            #rowConstructor
            // | qualifiedName '(' ASTERISK ')' filter_? over?                                        #functionCall
            // | qualifiedName '(' (setQuantifier? expression (',' expression)*)?
            //     (ORDER BY sortItem (',' sortItem)*)? ')' filter_? over?                            #functionCall
            // | identifier '->' expression                                                          #lambda
            // | TRY_CAST '(' expression AS type_ ')'                                                 #cast
            // | ARRAY '[' (expression (',' expression)*)? ']'                                       #arrayConstructor
            // | identifier                                                                          #columnReference
            // | SUBSTRING '(' valueExpression FROM valueExpression (FOR valueExpression)? ')'       #substring
            //
            // TODO: The disambiguation of several of these is incorrect
            // Currently we're preferring the special syntax form
            // when we could have a function call. This applies to:
            //    POSITION
            //    TRY_CAST
            //    SUBSTRING
            // Currently cannot parse functions calls with those names.
            // Need to verify if that's an issue.
            //
            // TODO: This could be tightened up a bit.
            | TK::Identifier
            | TK::QuotedIdentifier | TK::BackquotedIdentifier | TK::DigitIdentifier => true,
            _ => false,
            }
    }

    // booleanExpression
    // : valueExpression predicate[$valueExpression.ctx]?             #predicated
    // | NOT booleanExpression                                        #logicalNot
    // | left=booleanExpression operator=AND right=booleanExpression  #logicalBinary
    // | left=booleanExpression operator=OR right=booleanExpression   #logicalBinary
    fn parse_boolean_expression(&mut self) -> ParseTree<'a> {
        self.parse_or_expression()
    }

    fn parse_binary_expression(
        &mut self,
        peek_operator: Peeker<'a>,
        parse_operand: ElementParser<'a>,
    ) -> ParseTree<'a> {
        let left = parse_operand(self);
        self.parse_binary_expression_tail(left, peek_operator, parse_operand)
    }

    fn parse_binary_expression_tail(
        &mut self,
        left: ParseTree<'a>,
        peek_operator: Peeker<'a>,
        parse_operand: ElementParser<'a>,
    ) -> ParseTree<'a> {
        let mut left = left;
        while peek_operator(self) {
            let operator = self.eat_token();
            let right = parse_operand(self);
            left = parse_tree::binary_expression(left, operator, right);
        }
        left
    }

    // | left=booleanExpression operator=OR right=booleanExpression   #logicalBinary
    fn parse_or_expression(&mut self) -> ParseTree<'a> {
        self.parse_binary_expression(
            |parser| parser.peek_kind(TK::OR),
            |parser| parser.parse_and_expression(),
        )
    }

    fn parse_or_expression_tail(&mut self, and_expression: ParseTree<'a>) -> ParseTree<'a> {
        self.parse_binary_expression_tail(
            and_expression,
            |parser| parser.peek_kind(TK::OR),
            |parser| parser.parse_and_expression(),
        )
    }

    // | left=booleanExpression operator=AND right=booleanExpression  #logicalBinary
    fn parse_and_expression(&mut self) -> ParseTree<'a> {
        self.parse_binary_expression(
            |parser| parser.peek_kind(TK::AND),
            |parser| parser.parse_not_expression(),
        )
    }

    fn parse_and_expression_tail(&mut self, not_expression: ParseTree<'a>) -> ParseTree<'a> {
        self.parse_binary_expression_tail(
            not_expression,
            |parser| parser.peek_kind(TK::AND),
            |parser| parser.parse_not_expression(),
        )
    }

    // | NOT booleanExpression                                        #logicalNot
    fn parse_not_expression(&mut self) -> ParseTree<'a> {
        let not = self.eat_opt(TK::NOT);
        if !not.is_empty() {
            let operand = self.parse_not_expression();
            parse_tree::unary_expression(not, operand)
        } else {
            self.parse_predicated_expression()
        }
    }

    // : comparisonOperator right=valueExpression                            #comparison
    // | comparisonOperator comparisonQuantifier '(' query ')'               #quantifiedComparison
    fn parse_comparison_operator_suffix(&mut self, value: ParseTree<'a>) -> ParseTree<'a> {
        debug_assert!(self.peek_comparison_operator());
        let operator = self.eat_token();
        // TODO: Need better disambiguation between function_call
        // and comparison_quantifier((query) + 1)
        if self.peek_quantified_comparison() {
            let comparison_quantifier = self.eat_token();
            let (open_paren, query, close_paren) = self.parse_parenthesized_query();
            parse_tree::quantified_comparison(
                value,
                operator,
                comparison_quantifier,
                open_paren,
                query,
                close_paren,
            )
        } else {
            let right = self.parse_value_expression();
            parse_tree::binary_expression(value, operator, right)
        }
    }

    fn peek_comparison_operator(&mut self) -> bool {
        match self.peek() {
            TK::Equal
            | TK::LessGreater
            | TK::BangEqual
            | TK::OpenAngle
            | TK::CloseAngle
            | TK::LessEqual
            | TK::GreaterEqual => true,
            _ => false,
        }
    }

    // | IS NOT? NULL                                                        #nullPredicate
    // | IS NOT? DISTINCT FROM right=valueExpression                         #distinctFrom
    fn parse_is_suffix(&mut self, value: ParseTree<'a>) -> ParseTree<'a> {
        debug_assert!(self.peek_kind(TK::IS));
        let is = self.eat_token();
        let not_opt = self.eat_opt(TK::NOT);
        match self.peek() {
            TK::NULL => {
                let null = self.eat_token();
                parse_tree::null_predicate(value, is, not_opt, null)
            }
            TK::DISTINCT => {
                let distinct = self.eat_token();
                let from = self.eat(TK::FROM);
                let right = self.parse_value_expression();
                parse_tree::distinct_from(value, distinct, from, right)
            }
            _ => self.expected_error("NULL, DISTINCT"),
        }
    }

    // | NOT? BETWEEN lower=valueExpression AND upper=valueExpression        #between
    fn parse_between_suffix(
        &mut self,
        value: ParseTree<'a>,
        not_opt: ParseTree<'a>,
    ) -> ParseTree<'a> {
        let between = self.eat(TK::BETWEEN);
        let lower = self.parse_value_expression();
        let and = self.eat(TK::AND);
        let upper = self.parse_value_expression();
        parse_tree::between(value, not_opt, between, lower, and, upper)
    }

    // | NOT? LIKE pattern=valueExpression (ESCAPE escape=valueExpression)?  #like
    fn parse_like_suffix(&mut self, value: ParseTree<'a>, not_opt: ParseTree<'a>) -> ParseTree<'a> {
        let like = self.eat(TK::LIKE);
        let pattern = self.parse_value_expression();
        let escape_opt = self.eat_opt(TK::ESCAPE);
        let escape_value_opt = if escape_opt.is_empty() {
            self.eat_empty()
        } else {
            self.parse_value_expression()
        };
        parse_tree::like(value, not_opt, like, pattern, escape_opt, escape_value_opt)
    }

    // | NOT? IN '(' expression (',' expression)* ')'                        #inList
    // | NOT? IN '(' query ')'                                               #inSubquery
    fn parse_in_suffix(&mut self, value: ParseTree<'a>, not_opt: ParseTree<'a>) -> ParseTree<'a> {
        let in_ = self.eat(TK::IN);
        let expression_or_query = self.parse_row_constructor_or_subquery();
        if expression_or_query.is_parenthesized_expression() {
            let (open_paren, expression, close_paren) =
                expression_or_query.unbox_parenthesized_expression();
            let expressions = parse_tree::list(
                open_paren,
                vec![(
                    expression,
                    parse_tree::empty(TextRange::empty(close_paren.get_full_start())),
                )],
                close_paren,
            );
            parse_tree::in_list(value, not_opt, in_, expressions)
        } else if expression_or_query.is_row_constructor() {
            let (expressions,) = expression_or_query.unbox_row_constructor();
            parse_tree::in_list(value, not_opt, in_, expressions)
        } else {
            debug_assert!(expression_or_query.is_subquery_expression(),);
            let (open_paren, query, close_paren) = expression_or_query.unbox_subquery_expression();
            parse_tree::in_subquery(value, not_opt, in_, open_paren, query, close_paren)
        }
    }

    // : valueExpression predicate[$valueExpression.ctx]?             #predicated
    // predicate[ParserRuleContext value]
    // : comparisonOperator right=valueExpression                            #comparison
    // | comparisonOperator comparisonQuantifier '(' query ')'               #quantifiedComparison
    // | NOT? BETWEEN lower=valueExpression AND upper=valueExpression        #between
    // | NOT? IN '(' expression (',' expression)* ')'                        #inList
    // | NOT? IN '(' query ')'                                               #inSubquery
    // | NOT? LIKE pattern=valueExpression (ESCAPE escape=valueExpression)?  #like
    // | IS NOT? NULL                                                        #nullPredicate
    // | IS NOT? DISTINCT FROM right=valueExpression                         #distinctFrom
    fn parse_predicated_expression(&mut self) -> ParseTree<'a> {
        let value = self.parse_value_expression();
        self.parse_predicated_expression_tail(value)
    }

    fn parse_predicated_expression_tail(
        &mut self,
        value_expression: ParseTree<'a>,
    ) -> ParseTree<'a> {
        match self.peek() {
            TK::Equal
            | TK::LessGreater
            | TK::BangEqual
            | TK::OpenAngle
            | TK::CloseAngle
            | TK::LessEqual
            | TK::GreaterEqual => self.parse_comparison_operator_suffix(value_expression),
            TK::IS => self.parse_is_suffix(value_expression),
            _ => {
                let not_opt = self.eat_opt(TK::NOT);
                match self.peek() {
                    TK::BETWEEN => self.parse_between_suffix(value_expression, not_opt),
                    TK::IN => self.parse_in_suffix(value_expression, not_opt),
                    TK::LIKE => self.parse_like_suffix(value_expression, not_opt),
                    _ => {
                        if not_opt.is_empty() {
                            value_expression
                        } else {
                            self.expected_error("BETWEEN, IN, LIKE")
                        }
                    }
                }
            }
        }
    }

    fn peek_comparison_quantifier(&mut self) -> bool {
        match self.maybe_peek_predefined_name() {
            Some(PN::ALL) | Some(PN::SOME) | Some(PN::ANY) => true,
            _ => false,
        }
    }

    fn peek_quantified_comparison(&mut self) -> bool {
        self.peek_comparison_quantifier()
            && self.peek_kind_offset(TK::OpenParen, 1)
            && self.peek_query_primary_offset(2)
    }

    // valueExpression
    // : primaryExpression                                                                 #valueExpressionDefault
    // | valueExpression AT timeZoneSpecifier                                              #atTimeZone
    // | operator=(MINUS | PLUS) valueExpression                                           #arithmeticUnary
    // | left=valueExpression operator=(ASTERISK | SLASH | PERCENT) right=valueExpression  #arithmeticBinary
    // | left=valueExpression operator=(PLUS | MINUS) right=valueExpression                #arithmeticBinary
    // | left=valueExpression CONCAT right=valueExpression                                 #concatenation
    fn parse_value_expression(&mut self) -> ParseTree<'a> {
        self.parse_concat_expression()
    }

    // | left=valueExpression CONCAT right=valueExpression                                 #concatenation
    fn parse_concat_expression(&mut self) -> ParseTree<'a> {
        self.parse_binary_expression(
            |parser| parser.peek_kind(TK::BarBar),
            |parser| parser.parse_additive_expression(),
        )
    }

    fn parse_concat_expression_tail(
        &mut self,
        additive_expression: ParseTree<'a>,
    ) -> ParseTree<'a> {
        self.parse_binary_expression_tail(
            additive_expression,
            |parser| parser.peek_kind(TK::BarBar),
            |parser| parser.parse_additive_expression(),
        )
    }

    // | left=valueExpression operator=(PLUS | MINUS) right=valueExpression                #arithmeticBinary
    fn parse_additive_expression(&mut self) -> ParseTree<'a> {
        self.parse_binary_expression(
            |parser| parser.peek_additive_operator(),
            |parser| parser.parse_multiplicative_expression(),
        )
    }

    fn parse_additive_expression_tail(
        &mut self,
        multiplicative_expression: ParseTree<'a>,
    ) -> ParseTree<'a> {
        self.parse_binary_expression_tail(
            multiplicative_expression,
            |parser| parser.peek_additive_operator(),
            |parser| parser.parse_multiplicative_expression(),
        )
    }

    fn peek_additive_operator(&mut self) -> bool {
        match self.peek() {
            TK::Plus | TK::Minus => true,
            _ => false,
        }
    }

    // | left=valueExpression operator=(ASTERISK | SLASH | PERCENT) right=valueExpression  #arithmeticBinary
    fn parse_multiplicative_expression(&mut self) -> ParseTree<'a> {
        self.parse_binary_expression(
            |parser| parser.peek_multiplicative_operator(),
            |parser| parser.parse_arithmetic_unary_expression(),
        )
    }

    fn parse_multiplicative_expression_tail(
        &mut self,
        unary_expression: ParseTree<'a>,
    ) -> ParseTree<'a> {
        self.parse_binary_expression_tail(
            unary_expression,
            |parser| parser.peek_multiplicative_operator(),
            |parser| parser.parse_arithmetic_unary_expression(),
        )
    }

    fn peek_multiplicative_operator(&mut self) -> bool {
        match self.peek() {
            TK::Asterisk | TK::Slash | TK::Percent => true,
            _ => false,
        }
    }

    // | operator=(MINUS | PLUS) valueExpression                                           #arithmeticUnary
    fn parse_arithmetic_unary_expression(&mut self) -> ParseTree<'a> {
        if self.peek_additive_operator() {
            let operator = self.eat_token();
            let operand = self.parse_at_time_zone();
            parse_tree::unary_expression(operator, operand)
        } else {
            self.parse_at_time_zone()
        }
    }

    // | valueExpression AT timeZoneSpecifier                                              #atTimeZone
    // timeZoneSpecifier
    // : TIME ZONE interval  #timeZoneInterval
    // | TIME ZONE string    #timeZoneString
    fn parse_at_time_zone(&mut self) -> ParseTree<'a> {
        let left = self.parse_primary_expression();
        self.parse_at_time_zone_tail(left)
    }

    fn parse_at_time_zone_tail(&mut self, value_expression: ParseTree<'a>) -> ParseTree<'a> {
        let at = self.eat_predefined_name_opt(PN::AT);
        if at.is_empty() {
            value_expression
        } else {
            let time = self.eat_predefined_name(PN::TIME);
            let zone = self.eat_predefined_name(PN::ZONE);
            let specifier = if self.peek_predefined_name(PN::INTERVAL) {
                self.parse_interval()
            } else {
                self.parse_string()
            };
            parse_tree::at_time_zone(value_expression, at, time, zone, specifier)
        }
    }

    // qualifiedName
    // : identifier ('.' identifier)*
    fn parse_qualified_name(&mut self) -> ParseTree<'a> {
        // don't use parse_separated_list as period
        // is in the follow set of qualified_name
        let mut elements = Vec::new();
        let mut seperators = Vec::new();
        let start = self.eat_empty();
        elements.push(self.parse_identifier());
        while self.peek_kind(TK::Period) && self.peek_identifier_offset(1) {
            seperators.push(self.eat_opt(TK::Period));
            elements.push(self.parse_identifier());
        }
        seperators.push(self.eat_empty());
        let end = self.eat_empty();
        parse_tree::qualified_name(parse_tree::list(
            start,
            elements.into_iter().zip(seperators.into_iter()).collect(),
            end,
        ))
    }

    fn peek_qualified_name(&mut self) -> bool {
        self.peek_identifier()
    }

    fn parse_primary_prefix_expression(&mut self) -> ParseTree<'a> {
        match self.peek() {
            // : NULL                                                                                #nullLiteral
            TK::NULL => self.parse_literal(),
            // | DOUBLE_PRECISION string                                                             #typeConstructor
            TK::DoublePrecision => self.parse_type_constructor(),
            // | booleanValue                                                                        #booleanLiteral
            TK::TRUE | TK::FALSE => self.parse_literal(),
            // | number                                                                              #numericLiteral
            TK::Decimal | TK::Double | TK::Integer => self.parse_literal(),
            // | string                                                                              #stringLiteral
            TK::String | TK::UnicodeString => self.parse_literal(),
            // | BINARY_LITERAL                                                                      #binaryLiteral
            TK::BinaryLiteral => self.parse_literal(),
            // | '?'                                                                                 #parameter
            TK::Question => self.parse_parameter(),
            // // This is an extension to ANSI SQL, which considers EXISTS to be a <boolean expression>
            // | EXISTS '(' query ')'                                                                #exists
            TK::EXISTS => self.parse_exists(),
            // | CASE valueExpression whenClause+ (ELSE elseExpression=expression)? END              #simpleCase
            // | CASE whenClause+ (ELSE elseExpression=expression)? END                              #searchedCase
            TK::CASE => self.parse_case(),
            // | CAST '(' expression AS type_ ')'                                                     #cast
            TK::CAST => self.parse_cast(),
            // | name=CURRENT_DATE                                                                   #specialDateTimeFunction
            TK::CURRENT_DATE => self.parse_current_date(),
            // | name=CURRENT_TIME ('(' precision=INTEGER_VALUE ')')?                                #specialDateTimeFunction
            TK::CURRENT_TIME => self.parse_current_time(),
            // | name=CURRENT_TIMESTAMP ('(' precision=INTEGER_VALUE ')')?                           #specialDateTimeFunction
            TK::CURRENT_TIMESTAMP => self.parse_current_timestamp(),
            // | name=LOCALTIME ('(' precision=INTEGER_VALUE ')')?                                   #specialDateTimeFunction
            TK::LOCALTIME => self.parse_localtime(),
            // | name=LOCALTIMESTAMP ('(' precision=INTEGER_VALUE ')')?                              #specialDateTimeFunction
            TK::LOCALTIMESTAMP => self.parse_localtimestamp(),
            // | name=CURRENT_USER                                                                   #currentUser
            TK::CURRENT_USER => self.parse_current_user(),
            // | name=CURRENT_PATH                                                                   #currentPath
            TK::CURRENT_PATH => self.parse_current_path(),
            // | NORMALIZE '(' valueExpression (',' normalForm)? ')'                                 #normalize
            TK::NORMALIZE => self.parse_normalize(),
            // | EXTRACT '(' identifier FROM valueExpression ')'                                     #extract
            TK::EXTRACT => self.parse_extract(),
            // | GROUPING '(' (qualifiedName (',' qualifiedName)*)? ')'                              #groupingOperation
            TK::GROUPING => self.parse_grouping(),
            // | configureExpression                                                                 #conf
            TK::CONFIGURE => self.parse_configure_expression(),

            // | '(' expression (',' expression)+ ')'                                                #rowConstructor
            // | '(' (identifier (',' identifier)*)? ')' '->' expression                             #lambda
            // | '(' query ')'                                                                       #subqueryExpression
            // | '(' expression ')'                                                                  #parenthesizedExpression
            TK::OpenParen => {
                if self.peek_lambda() {
                    self.parse_lambda()
                } else {
                    self.parse_row_constructor_or_subquery()
                }
            }

            // | interval                                                                            #intervalLiteral
            // | identifier string                                                                   #typeConstructor
            // | POSITION '(' valueExpression IN valueExpression ')'                                 #position
            // | ROW '(' expression (',' expression)* ')'                                            #rowConstructor
            // | qualifiedName '(' ASTERISK ')' filter_? over?                                        #functionCall
            // | qualifiedName '(' (setQuantifier? expression (',' expression)*)?
            //     (ORDER BY sortItem (',' sortItem)*)? ')' filter_? over?                            #functionCall
            // | identifier '->' expression                                                          #lambda
            // | TRY_CAST '(' expression AS type_ ')'                                                 #cast
            // | ARRAY '[' (expression (',' expression)*)? ']'                                       #arrayConstructor
            // | identifier                                                                          #columnReference
            // | SUBSTRING '(' valueExpression FROM valueExpression (FOR valueExpression)? ')'       #substring
            //
            // TODO: The disambiguation of several of these is incorrect
            // Currently we're prefering the special syntgax form
            // when we could have a function call. This applies to:
            //    POSITION
            //    TRY_CAST
            //    SUBSTRING
            // Currently cannot parse functions calls with those names.
            // Need to verify if that's an issue.
            TK::Identifier => {
                if let Some(name) = self.maybe_peek_predefined_name() {
                    match name {
                        PN::INTERVAL => {
                            if self.peek_interval() {
                                return self.parse_interval();
                            }
                        }
                        PN::POSITION => {
                            if self.peek_position() {
                                return self.parse_position();
                            }
                        }
                        PN::ROW => {
                            if self.peek_row_constructor() {
                                return self.parse_row_constructor();
                            }
                        }
                        PN::TRY_CAST => {
                            if self.peek_try_cast() {
                                return self.parse_try_cast();
                            }
                        }
                        PN::ARRAY => {
                            if self.peek_array_constructor() {
                                return self.parse_array_constructor();
                            }
                        }
                        PN::SUBSTRING => {
                            if self.peek_substring() {
                                return self.parse_substring();
                            }
                        }
                        _ => (),
                    }
                }
                // | identifier string                                                                   #typeConstructor
                // | qualifiedName '(' ASTERISK ')' filter_? over?                                        #functionCall
                // | qualifiedName '(' (setQuantifier? expression (',' expression)*)?
                //     (ORDER BY sortItem (',' sortItem)*)? ')' filter_? over?                            #functionCall
                // | identifier '->' expression                                                          #lambda
                // | identifier                                                                          #columnReference
                self.parse_identifier_start_expression()
            }
            // | identifier string                                                                   #typeConstructor
            // | qualifiedName '(' ASTERISK ')' filter_? over?                                        #functionCall
            // | qualifiedName '(' (setQuantifier? expression (',' expression)*)?
            // | identifier '->' expression                                                          #lambda
            // | identifier                                                                          #columnReference
            TK::QuotedIdentifier | TK::BackquotedIdentifier | TK::DigitIdentifier => {
                self.parse_identifier_start_expression()
            }
            _ => self.expected_error("Expected expression."),
        }
    }

    fn parse_primary_expression(&mut self) -> ParseTree<'a> {
        let operand = self.parse_primary_prefix_expression();
        self.parse_primary_expression_tail(operand)
    }

    fn parse_primary_expression_tail(
        &mut self,
        primary_expression: ParseTree<'a>,
    ) -> ParseTree<'a> {
        let mut result = primary_expression;
        loop {
            // suffixes
            match self.peek() {
                // | base=primaryExpression '.' fieldName=identifier                                     #dereference
                TK::Period => {
                    let period = self.eat(TK::Period);
                    let field_name = self.parse_identifier();
                    result = parse_tree::dereference(result, period, field_name)
                }
                // | value=primaryExpression '[' index=valueExpression ']'                               #subscript
                TK::OpenSquare => {
                    let open_square = self.eat(TK::OpenSquare);
                    let index = self.parse_value_expression();
                    let close_square = self.eat(TK::CloseSquare);
                    result = parse_tree::subscript(result, open_square, index, close_square)
                }
                _ => return result,
            }
        }
    }

    fn parse_paren_expression_tail(&mut self, paren_expression: ParseTree<'a>) -> ParseTree<'a> {
        let primary_expression = self.parse_primary_expression_tail(paren_expression);
        let at_time_zone = self.parse_at_time_zone_tail(primary_expression);
        let multitplicative = self.parse_multiplicative_expression_tail(at_time_zone);
        let additive = self.parse_additive_expression_tail(multitplicative);
        let concat = self.parse_concat_expression_tail(additive);
        let predicate = self.parse_predicated_expression_tail(concat);
        let and = self.parse_and_expression_tail(predicate);
        let or = self.parse_or_expression_tail(and);
        or
    }

    // | '(' (identifier (',' identifier)*)? ')' '->' expression                             #lambda
    fn peek_lambda(&mut self) -> bool {
        if self.peek_kind(TK::OpenParen) {
            let mut offset = 1;
            if self.peek_identifier_offset(offset) {
                offset += 1;
                while self.peek_kind_offset(TK::Comma, offset) {
                    offset += 1;
                    if self.peek_identifier_offset(offset) {
                        offset += 1;
                    } else {
                        return false;
                    }
                }
            }
            self.peek_kind_offset(TK::CloseParen, offset)
                && self.peek_kind_offset(TK::Arrow, offset + 1)
        } else {
            false
        }
    }

    fn parse_lambda(&mut self) -> ParseTree<'a> {
        let parameters = self.parse_delimited_separated_list_opt(
            TK::OpenParen,
            TK::Comma,
            |parser| parser.peek_identifier(),
            |parser| parser.parse_identifier(),
            TK::CloseParen,
        );
        let array = self.eat(TK::Arrow);
        let body = self.parse_expression();
        parse_tree::lambda(parameters, array, body)
    }

    fn paren_expression_or_query_to_expression(
        &mut self,
        open_paren: ParseTree<'a>,
        expression_or_query: ParseTree<'a>,
        close_paren: ParseTree<'a>,
    ) -> ParseTree<'a> {
        if expression_or_query.is_query() || expression_or_query.is_query_no_with() {
            parse_tree::subquery_expression(open_paren, expression_or_query, close_paren)
        } else if expression_or_query.is_expression_or_query() {
            parse_tree::parenthesized_expression(
                open_paren,
                self.expression_or_query_to_expression(expression_or_query),
                close_paren,
            )
        } else {
            // TODO: debug_assert!(expression_or_query.is_any_expression());
            parse_tree::parenthesized_expression(open_paren, expression_or_query, close_paren)
        }
    }

    fn expression_or_query_to_expression(
        &mut self,
        expression_or_query: ParseTree<'a>,
    ) -> ParseTree<'a> {
        if expression_or_query.is_query() || expression_or_query.is_query_no_with() {
            self.add_error_of_tree(&expression_or_query, "Expected expression, found query.");
            expression_or_query
        } else if expression_or_query.is_expression_or_query() {
            let (open_paren, expression_or_query, close_paren) =
                expression_or_query.unbox_expression_or_query();
            self.paren_expression_or_query_to_expression(
                open_paren,
                expression_or_query,
                close_paren,
            )
        } else {
            // TODO: debug_assert!(expression_or_query.is_any_expression())
            expression_or_query
        }
    }

    // yields one of:
    //   one of several expression trees:
    //   query - possibly with or without a with clause
    //   query_no_with
    //   expression_or_query
    fn parse_expression_or_query(&mut self) -> ParseTree<'a> {
        if self.peek_kind(TK::OpenParen) {
            let open_paren = self.eat(TK::OpenParen);
            let expression_or_query = self.parse_expression_or_query();
            if self.peek_kind(TK::Comma) {
                let row_constructor =
                    self.parse_row_constructor_tail(open_paren, expression_or_query);
                self.parse_paren_expression_tail(row_constructor)
            } else {
                let close_paren = self.eat(TK::CloseParen);
                let must_be_subquery_expression = expression_or_query.is_query()
                    && !expression_or_query.as_query().with.is_empty();
                if must_be_subquery_expression {
                    parse_tree::subquery_expression(open_paren, expression_or_query, close_paren)
                } else if {
                    let can_be_query_primary = !must_be_subquery_expression
                        && (expression_or_query.is_query_no_with()
                            || expression_or_query.is_query()
                            || expression_or_query.is_expression_or_query());
                    can_be_query_primary
                } {
                    let must_be_query_tail = self.peek_query_primary_follow();
                    if must_be_query_tail {
                        let subquery =
                            parse_tree::subquery(open_paren, expression_or_query, close_paren);
                        // this yields a query_no_with
                        self.parse_query_primary_tail(subquery)
                    } else if self.peek_kind(TK::CloseParen) {
                        // we have a query which can be consumed as either
                        // a subquery or a subquery_expression...
                        // make the decision up the tree.
                        parse_tree::expression_or_query(
                            open_paren,
                            expression_or_query,
                            close_paren,
                        )
                    } else {
                        // we have a parenthesized query, with what looks like an expression tail
                        // afterwards
                        let subquery_expression = parse_tree::subquery_expression(
                            open_paren,
                            expression_or_query,
                            close_paren,
                        );
                        self.parse_paren_expression_tail(subquery_expression)
                    }
                } else {
                    // we have an expression
                    self.parse_paren_expression_tail(parse_tree::parenthesized_expression(
                        open_paren,
                        expression_or_query,
                        close_paren,
                    ))
                }
            }
        } else if self.peek_query_offset(0) {
            // yields a query
            self.parse_query()
        } else {
            self.parse_expression()
        }
    }

    // | '(' expression (',' expression)+ ')'                                                #rowConstructor
    // | '(' query ')'                                                                       #subqueryExpression
    // | '(' expression ')'                                                                  #parenthesizedExpression
    fn parse_row_constructor_or_subquery(&mut self) -> ParseTree<'a> {
        let open_paren = self.eat(TK::OpenParen);
        let expression_or_query = self.parse_expression_or_query();
        if self.peek_kind(TK::Comma) {
            self.parse_row_constructor_tail(open_paren, expression_or_query)
        } else {
            // either expression or query is permitted
            let close_paren = self.eat(TK::CloseParen);
            self.paren_expression_or_query_to_expression(
                open_paren,
                expression_or_query,
                close_paren,
            )
        }
    }

    /// parse expression_list tail and return a row constructor
    fn parse_row_constructor_tail(
        &mut self,
        open_paren: ParseTree<'a>,
        expression_or_query: ParseTree<'a>,
    ) -> ParseTree<'a> {
        let comma = self.eat(TK::Comma);
        let mut elements_tail =
            self.parse_separated_list_elements(TK::Comma, |parser| parser.parse_expression());
        let close_paren = self.eat(TK::CloseParen);
        let mut elements = Vec::with_capacity(elements_tail.len() + 1);
        // validate that expression_or_query is actually an expression.
        let expression = self.expression_or_query_to_expression(expression_or_query);
        elements.push((expression, comma));
        elements.append(&mut elements_tail);
        parse_tree::row_constructor(parse_tree::list(open_paren, elements, close_paren))
    }

    // | POSITION '(' valueExpression IN valueExpression ')'                                 #position
    fn peek_position(&mut self) -> bool {
        self.peek_predefined_name(PN::POSITION) && self.peek_kind_offset(TK::OpenParen, 1)
    }

    fn parse_position(&mut self) -> ParseTree<'a> {
        let position = self.eat_predefined_name(PN::POSITION);
        let open_paren = self.eat(TK::OpenParen);
        let value = self.parse_value_expression();
        let in_ = self.eat(TK::IN);
        let target = self.parse_value_expression();
        let close_paren = self.eat(TK::CloseParen);
        parse_tree::position(position, open_paren, value, in_, target, close_paren)
    }

    // interval
    // : INTERVAL sign=(PLUS | MINUS)? (string | configureExpression) from_=intervalField (TO to=intervalField)?
    fn peek_interval(&mut self) -> bool {
        // TODO: must peek all the way to the interval_field
        // to ensure diambiguation with additive_binary if sign is present
        // TODO: must peek to interval field if no-sign present,
        // string is present, to disambiguate with type constructor
        self.peek_predefined_name(PN::INTERVAL)
            && match self.peek_offset(1) {
                TK::Plus
                | TK::Minus
                | TK::String
                | TK::UnicodeString
                | TK::Identifier
                | TK::CONFIGURE => true,
                _ => false,
            }
    }

    fn parse_interval(&mut self) -> ParseTree<'a> {
        let interval = self.eat_predefined_name(PN::INTERVAL);
        let sign_opt = self.parse_sign_opt();
        let value = if self.peek_string() {
            self.parse_string()
        } else {
            self.parse_configure_expression()
        };
        let from = self.parse_interval_field();
        let to_kw_opt = self.eat_predefined_name_opt(PN::TO);
        let to = if to_kw_opt.is_empty() {
            self.eat_empty()
        } else {
            self.parse_interval_field()
        };
        parse_tree::interval(interval, sign_opt, value, from, to_kw_opt, to)
    }

    fn parse_sign_opt(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TK::Plus | TK::Minus => self.eat_token(),
            _ => self.eat_empty(),
        }
    }

    // intervalField
    // : YEAR | MONTH | DAY | HOUR | MINUTE | SECOND
    fn peek_interval_field_offset(&mut self, offset: usize) -> bool {
        match self.maybe_peek_predefined_name_offset(offset) {
            Some(PN::YEAR) | Some(PN::MONTH) | Some(PN::DAY) | Some(PN::HOUR)
            | Some(PN::MINUTE) | Some(PN::SECOND) => true,
            _ => false,
        }
    }

    fn parse_interval_field(&mut self) -> ParseTree<'a> {
        if self.peek_interval_field_offset(0) {
            self.eat_token()
        } else {
            self.expected_error("interval field")
        }
    }

    // | ROW '(' expression (',' expression)* ')'                                            #rowConstructor
    fn peek_row_constructor(&mut self) -> bool {
        self.peek_predefined_name(PN::ROW) && self.peek_kind_offset(TK::OpenParen, 1)
    }

    fn parse_row_constructor(&mut self) -> ParseTree<'a> {
        let row = self.eat_predefined_name(PN::ROW);
        let elements =
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_expression());
        parse_tree::row(row, elements)
    }

    // | TRY_CAST '(' expression AS type_ ')'                                                 #cast
    fn peek_try_cast(&mut self) -> bool {
        self.peek_predefined_name(PN::TRY_CAST) && self.peek_kind_offset(TK::OpenParen, 1)
    }

    fn parse_try_cast(&mut self) -> ParseTree<'a> {
        let try_cast = self.eat_predefined_name(PN::TRY_CAST);
        let open_paren = self.eat(TK::OpenParen);
        let value = self.parse_expression();
        let as_ = self.eat(TK::AS);
        let type_ = self.parse_type();
        let close_paren = self.eat(TK::CloseParen);
        parse_tree::try_cast(try_cast, open_paren, value, as_, type_, close_paren)
    }

    // | ARRAY '[' (expression (',' expression)*)? ']'                                       #arrayConstructor
    fn peek_array_constructor(&mut self) -> bool {
        self.peek_predefined_name(PN::ARRAY) && self.peek_kind_offset(TK::OpenSquare, 1)
    }

    fn parse_array_constructor(&mut self) -> ParseTree<'a> {
        let array = self.eat_predefined_name(PN::ARRAY);
        let elements = self.parse_delimited_separated_list_opt(
            TK::OpenSquare,
            TK::Comma,
            |parser| parser.peek_expression(),
            |parser| parser.parse_expression(),
            TK::CloseSquare,
        );
        parse_tree::array(array, elements)
    }

    // configureExpression
    //     : CONFIGURE '(' identifier ',' configure_value_ ')'
    fn peek_configure_expression(&mut self) -> bool {
        self.peek_kind(TK::CONFIGURE)
    }

    fn parse_configure_expression(&mut self) -> ParseTree<'a> {
        let configure = self.eat(TK::CONFIGURE);
        let open_paren = self.eat(TK::OpenParen);
        let identifier = self.parse_identifier();
        let comma = self.eat(TK::Comma);
        let value = self.parse_configure_value();
        let close_paren = self.eat(TK::CloseParen);
        parse_tree::configure_expression(
            configure,
            open_paren,
            identifier,
            comma,
            value,
            close_paren,
        )
    }

    // configure_value_
    // : string | number | booleanValue
    fn parse_configure_value(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TK::String
            | TK::UnicodeString
            | TK::Decimal
            | TK::Double
            | TK::Integer
            | TK::TRUE
            | TK::FALSE => self.parse_literal(),
            _ => self.expected_error("configure value"),
        }
    }

    // | SUBSTRING '(' valueExpression FROM valueExpression (FOR valueExpression)? ')'       #substring
    fn peek_substring(&mut self) -> bool {
        self.peek_predefined_name(PN::SUBSTRING) && self.peek_kind_offset(TK::OpenParen, 1)
    }

    fn parse_substring(&mut self) -> ParseTree<'a> {
        let substring = self.eat_predefined_name(PN::SUBSTRING);
        let open_paren = self.eat(TK::OpenParen);
        let value = self.parse_value_expression();
        let from = self.eat(TK::FROM);
        let from_value = self.parse_value_expression();
        let for_opt = self.eat_opt(TK::FOR);
        let for_value = if for_opt.is_empty() {
            self.eat_empty()
        } else {
            self.parse_value_expression()
        };
        let close_paren = self.eat(TK::CloseParen);
        parse_tree::substring(
            substring,
            open_paren,
            value,
            from,
            from_value,
            for_opt,
            for_value,
            close_paren,
        )
    }

    // | '(' query ')'                                                                       #subqueryExpression
    fn parse_subquery_expression(&mut self) -> ParseTree<'a> {
        let (open_paren, query, close_paren) = self.parse_parenthesized_query();
        parse_tree::subquery_expression(open_paren, query, close_paren)
    }

    // | GROUPING '(' (qualifiedName (',' qualifiedName)*)? ')'                              #groupingOperation
    fn parse_grouping(&mut self) -> ParseTree<'a> {
        let grouping = self.eat(TK::GROUPING);
        let groups = self.parse_delimited_separated_list_opt(
            TK::OpenParen,
            TK::Comma,
            |parser| parser.peek_qualified_name(),
            |parser| parser.parse_qualified_name(),
            TK::CloseParen,
        );
        parse_tree::grouping(grouping, groups)
    }

    // | EXTRACT '(' identifier FROM valueExpression ')'                                     #extract
    fn parse_extract(&mut self) -> ParseTree<'a> {
        let extract = self.eat(TK::EXTRACT);
        let open_paren = self.eat(TK::OpenParen);
        let identifier = self.parse_identifier();
        let from = self.eat(TK::FROM);
        let value = self.parse_value_expression();
        let close_paren = self.eat(TK::CloseParen);
        parse_tree::extract(extract, open_paren, identifier, from, value, close_paren)
    }

    // | name=CURRENT_PATH                                                                   #currentPath
    fn parse_current_path(&mut self) -> ParseTree<'a> {
        self.parse_literal()
    }

    // | name=CURRENT_USER                                                                   #currentUser
    fn parse_current_user(&mut self) -> ParseTree<'a> {
        self.parse_literal()
    }

    // | name=CURRENT_DATE                                                                   #specialDateTimeFunction
    fn parse_current_date(&mut self) -> ParseTree<'a> {
        self.parse_literal()
    }

    // | name=CURRENT_TIME ('(' precision=INTEGER_VALUE ')')?                                #specialDateTimeFunction
    fn parse_current_time(&mut self) -> ParseTree<'a> {
        let current_time = self.eat(TK::CURRENT_TIME);
        let (open_paren, precision, close_paren) = if self.peek_kind(TK::OpenParen) {
            self.parse_parenthesized(|parser| parser.eat(TK::Integer))
        } else {
            (self.eat_empty(), self.eat_empty(), self.eat_empty())
        };
        parse_tree::current_time(current_time, open_paren, precision, close_paren)
    }

    // | name=CURRENT_TIMESTAMP ('(' precision=INTEGER_VALUE ')')?                           #specialDateTimeFunction
    fn parse_current_timestamp(&mut self) -> ParseTree<'a> {
        let current_timestamp = self.eat(TK::CURRENT_TIMESTAMP);
        let (open_paren, precision, close_paren) = if self.peek_kind(TK::OpenParen) {
            self.parse_parenthesized(|parser| parser.eat(TK::Integer))
        } else {
            (self.eat_empty(), self.eat_empty(), self.eat_empty())
        };
        parse_tree::current_timestamp(current_timestamp, open_paren, precision, close_paren)
    }

    // | NORMALIZE '(' valueExpression (',' normalForm)? ')'                                 #normalize
    fn parse_normalize(&mut self) -> ParseTree<'a> {
        let normalize = self.eat(TK::NORMALIZE);
        let open_paren = self.eat(TK::OpenParen);
        let value = self.parse_value_expression();
        let comma_opt = self.eat_opt(TK::Comma);
        let normal_form = if comma_opt.is_empty() {
            self.eat_empty()
        } else {
            self.parse_normal_form()
        };
        let close_paren = self.eat(TK::CloseParen);
        parse_tree::normalize(
            normalize,
            open_paren,
            value,
            comma_opt,
            normal_form,
            close_paren,
        )
    }

    // normalForm
    // : NFD | NFC | NFKD | NFKC
    fn parse_normal_form(&mut self) -> ParseTree<'a> {
        match self.maybe_peek_predefined_name() {
            Some(PN::NFD) | Some(PN::NFC) | Some(PN::NFKD) | Some(PN::NFKC) => self.eat_token(),
            _ => self.expected_error("normal form"),
        }
    }

    // | name=LOCALTIMESTAMP ('(' precision=INTEGER_VALUE ')')?                              #specialDateTimeFunction
    fn parse_localtimestamp(&mut self) -> ParseTree<'a> {
        let localtimestamp = self.eat(TK::LOCALTIMESTAMP);
        let (open_paren, precision, close_paren) = if self.peek_kind(TK::OpenParen) {
            self.parse_parenthesized(|parser| parser.eat(TK::Integer))
        } else {
            (self.eat_empty(), self.eat_empty(), self.eat_empty())
        };
        parse_tree::localtimestamp(localtimestamp, open_paren, precision, close_paren)
    }

    // | name=LOCALTIME ('(' precision=INTEGER_VALUE ')')?                                   #specialDateTimeFunction
    fn parse_localtime(&mut self) -> ParseTree<'a> {
        let localtime = self.eat(TK::LOCALTIME);
        let (open_paren, precision, close_paren) = if self.peek_kind(TK::OpenParen) {
            self.parse_parenthesized(|parser| parser.eat(TK::Integer))
        } else {
            (self.eat_empty(), self.eat_empty(), self.eat_empty())
        };
        parse_tree::localtime(localtime, open_paren, precision, close_paren)
    }

    // | CAST '(' expression AS type_ ')'                                                     #cast
    fn parse_cast(&mut self) -> ParseTree<'a> {
        let cast = self.eat(TK::CAST);
        let open_paren = self.eat(TK::OpenParen);
        let value = self.parse_expression();
        let as_ = self.eat(TK::AS);
        let type_ = self.parse_type();
        let close_paren = self.eat(TK::CloseParen);
        parse_tree::cast(cast, open_paren, value, as_, type_, close_paren)
    }

    // | CASE valueExpression whenClause+ (ELSE elseExpression=expression)? END              #simpleCase
    // | CASE whenClause+ (ELSE elseExpression=expression)? END                              #searchedCase
    fn parse_case(&mut self) -> ParseTree<'a> {
        let case = self.eat(TK::CASE);
        let value_opt = if self.peek_when_clause() {
            self.eat_empty()
        } else {
            self.parse_expression()
        };
        let when_clauses = self.parse_list(
            |parser| parser.peek_when_clause(),
            |parser| parser.parse_when_clause(),
        );
        let else_opt = self.eat_opt(TK::ELSE);
        let default = if else_opt.is_empty() {
            self.eat_empty()
        } else {
            self.parse_expression()
        };
        let end = self.eat(TK::END);
        parse_tree::case(case, value_opt, when_clauses, else_opt, default, end)
    }

    // whenClause
    // : WHEN condition=expression THEN result=expression
    fn parse_when_clause(&mut self) -> ParseTree<'a> {
        let when = self.eat(TK::WHEN);
        let condition = self.parse_expression();
        let then = self.eat(TK::THEN);
        let result = self.parse_expression();
        parse_tree::when_clause(when, condition, then, result)
    }

    fn peek_when_clause(&mut self) -> bool {
        self.peek_kind(TK::WHEN)
    }

    // | EXISTS '(' query ')'                                                                #exists
    fn parse_exists(&mut self) -> ParseTree<'a> {
        let exists = self.eat(TK::EXISTS);
        let (open_paren, query, close_paren) = self.parse_parenthesized_query();
        parse_tree::exists(exists, open_paren, query, close_paren)
    }

    // | '?'                                                                                 #parameter
    fn parse_parameter(&mut self) -> ParseTree<'a> {
        self.eat(TK::Question)
    }

    fn parse_literal(&mut self) -> ParseTree<'a> {
        parse_tree::literal(self.eat_token())
    }

    // | identifier string                                                                   #typeConstructor
    // | DOUBLE_PRECISION string                                                             #typeConstructor
    fn parse_type_constructor(&mut self) -> ParseTree<'a> {
        let type_ = if self.peek_kind(TK::DoublePrecision) {
            self.eat_token()
        } else {
            self.parse_identifier()
        };
        let value = self.parse_string();
        parse_tree::type_constructor(type_, value)
    }

    // | identifier string                                                                   #typeConstructor
    // | qualifiedName '(' ASTERISK ')' filter_? over?                                        #functionCall
    // | qualifiedName '(' (setQuantifier? expression (',' expression)*)?
    //     (ORDER BY sortItem (',' sortItem)*)? ')' filter_? over?                            #functionCall
    // | identifier '->' expression                                                          #lambda
    // | identifier                                                                          #columnReference
    fn parse_identifier_start_expression(&mut self) -> ParseTree<'a> {
        match self.peek_offset(1) {
            TK::Arrow => self.parse_parenless_lambda(),
            TK::String | TK::UnicodeString => self.parse_type_constructor(),
            _ => {
                if self.peek_function_call() {
                    self.parse_function_call()
                } else {
                    parse_tree::identifier(self.eat_token())
                }
            }
        }
    }

    fn parse_parenless_lambda(&mut self) -> ParseTree<'a> {
        let parameter = self.eat_token();
        let arrow = self.eat(TK::Arrow);
        let body = self.parse_expression();
        parse_tree::lambda(parameter, arrow, body)
    }

    // | qualifiedName '(' ASTERISK ')' filter_? over?                                        #functionCall
    // | qualifiedName '(' (setQuantifier? expression (',' expression)*)?
    //     (ORDER BY sortItem (',' sortItem)*)? ')' filter_? over?                            #functionCall
    fn peek_function_call(&mut self) -> bool {
        let mut offset = 1;
        while self.peek_kind_offset(TK::Period, offset) {
            offset += 1;
            if self.peek_identifier_offset(offset) {
                offset += 1;
            } else {
                return false;
            }
        }
        self.peek_kind_offset(TK::OpenParen, offset)
    }

    fn parse_function_call(&mut self) -> ParseTree<'a> {
        let name = self.parse_qualified_name();
        let open_paren = self.eat(TK::OpenParen);
        if self.peek_kind(TK::Asterisk) {
            let set_quantifier_opt = self.eat_empty();
            let arguments = self.eat(TK::Asterisk);
            let order_by_opt = self.eat_empty();
            let close_paren = self.eat(TK::CloseParen);
            let filter_opt = self.parse_filter_opt();
            let null_treatment_opt = self.eat_empty();
            let over_opt = self.parse_over_opt();
            parse_tree::function_call(
                name,
                open_paren,
                set_quantifier_opt,
                arguments,
                order_by_opt,
                close_paren,
                filter_opt,
                null_treatment_opt,
                over_opt,
            )
        } else {
            let set_quantifier_opt = self.parse_set_quantifier_opt(|parser, offset| {
                parser.peek_kind_offset(TK::CloseParen, offset)
                    || parser.peek_expression_offset(offset)
                    || parser.peek_kind_offset(TK::ORDER, offset)
            });
            let arguments = if set_quantifier_opt.is_empty() {
                self.parse_comma_separated_list_opt(
                    |parser| parser.peek_expression(),
                    |parser| parser.parse_expression(),
                )
            } else {
                self.parse_comma_separated_list(|parser| parser.parse_expression())
            };
            let order_by_opt = self.parse_order_by_opt();
            let close_paren = self.eat(TK::CloseParen);
            let filter_opt = self.parse_filter_opt();
            let null_treatment_opt = self.parse_null_treatment_opt();
            let over_opt = if null_treatment_opt.is_empty() {
                self.parse_over_opt()
            } else {
                self.parse_over()
            };
            parse_tree::function_call(
                name,
                open_paren,
                set_quantifier_opt,
                arguments,
                order_by_opt,
                close_paren,
                filter_opt,
                null_treatment_opt,
                over_opt,
            )
        }
    }

    // filter
    // : FILTER '(' WHERE booleanExpression ')'
    fn parse_filter_opt(&mut self) -> ParseTree<'a> {
        if self.peek_filter() {
            let filter = self.eat_predefined_name(PN::FILTER);
            let open_paren = self.eat(TK::OpenParen);
            let where_ = self.eat(TK::WHERE);
            let predicate = self.parse_boolean_expression();
            let close_paren = self.eat(TK::CloseParen);
            parse_tree::filter(filter, open_paren, where_, predicate, close_paren)
        } else {
            self.eat_empty()
        }
    }

    fn peek_filter(&mut self) -> bool {
        self.peek_predefined_name(PN::FILTER) && self.peek_kind_offset(TK::OpenParen, 1)
    }

    // over
    // : OVER '('
    //     (PARTITION BY partition+=expression (',' partition+=expression)*)?
    //     (ORDER BY sortItem (',' sortItem)*)?
    //     windowFrame?
    //   ')'
    fn parse_over_opt(&mut self) -> ParseTree<'a> {
        if self.peek_over() {
            self.parse_over()
        } else {
            self.eat_empty()
        }
    }

    fn parse_over(&mut self) -> ParseTree<'a> {
        let over = self.eat_predefined_name(PN::OVER);
        let open_paren = self.eat(TK::OpenParen);
        let partition_opt = self.eat_predefined_name_opt(PN::PARTITION);
        let (by, partitions) = if partition_opt.is_empty() {
            (self.eat_empty(), self.eat_empty())
        } else {
            (
                self.eat(TK::BY),
                self.parse_comma_separated_list(|parser| parser.parse_expression()),
            )
        };
        let order_by_opt = self.parse_order_by_opt();
        let window_frame = self.parse_window_frame_opt();
        let close_paren = self.eat(TK::CloseParen);
        parse_tree::over(
            over,
            open_paren,
            partition_opt,
            by,
            partitions,
            order_by_opt,
            window_frame,
            close_paren,
        )
    }

    fn peek_over(&mut self) -> bool {
        self.peek_predefined_name(PN::OVER) && self.peek_kind_offset(TK::OpenParen, 1)
    }

    // nullTreatment
    // : IGNORE NULLS
    // | RESPECT NULLS
    fn parse_null_treatment_opt(&mut self) -> ParseTree<'a> {
        if self.peek_null_treatment() {
            let treatment = self.eat(TK::Identifier);
            let nulls = self.eat_predefined_name(PN::NULLS);
            parse_tree::null_treatment(treatment, nulls)
        } else {
            self.eat_empty()
        }
    }

    fn peek_null_treatment(&mut self) -> bool {
        (self.peek_predefined_name(PN::IGNORE) || self.peek_predefined_name(PN::RESPECT))
        && self.peek_predefined_name_offset(PN::NULLS, 1)
        // null treatement must be followed by OVER
        && self.peek_predefined_name_offset(PN::OVER, 2)
    }

    // windowFrame
    // : frameType=RANGE startBound=frameBound
    // | frameType=ROWS startBound=frameBound
    // | frameType=RANGE BETWEEN startBound=frameBound AND end=frameBound
    // | frameType=ROWS BETWEEN startBound=frameBound AND end=frameBound
    fn parse_window_frame_opt(&mut self) -> ParseTree<'a> {
        if self.peek_predefined_name(PN::RANGE) || self.peek_predefined_name(PN::ROWS) {
            let frame_type = self.eat_token();
            let between_opt = self.eat_opt(TK::BETWEEN);
            let start = self.parse_frame_bound();
            let (and, end) = if between_opt.is_empty() {
                (self.eat_empty(), self.eat_empty())
            } else {
                (self.eat(TK::AND), self.parse_frame_bound())
            };
            parse_tree::window_frame(frame_type, between_opt, start, and, end)
        } else {
            self.eat_empty()
        }
    }

    // frameBound
    // : UNBOUNDED boundType=PRECEDING                 #unboundedFrame
    // | UNBOUNDED boundType=FOLLOWING                 #unboundedFrame
    // | CURRENT ROW                                   #currentRowBound
    // | expression boundType=(PRECEDING | FOLLOWING)  #boundedFrame // expression should be unsignedLiteral
    fn parse_frame_bound(&mut self) -> ParseTree<'a> {
        if self.peek_predefined_name(PN::UNBOUNDED)
            && (self.peek_predefined_name_offset(PN::PRECEDING, 1)
                || self.peek_predefined_name_offset(PN::FOLLOWING, 1))
        {
            parse_tree::unbounded_frame(self.eat_token(), self.eat_token())
        } else if self.peek_predefined_name_offset(PN::CURRENT, 0)
            && self.peek_predefined_name_offset(PN::ROW, 1)
        {
            parse_tree::current_row_bound(self.eat_token(), self.eat_token())
        } else {
            let bound = self.parse_expression();
            let bound_type = self.parse_bound_type();
            parse_tree::bounded_frame(bound, bound_type)
        }
    }

    fn parse_bound_type(&mut self) -> ParseTree<'a> {
        match self.maybe_peek_predefined_name() {
            Some(PN::PRECEDING) | Some(PN::FOLLOWING) => self.eat_token(),
            _ => self.expected_error("PRECEDING, FOLLOWING"),
        }
    }

    // string
    // : STRING                                #basicStringLiteral
    // | UNICODE_STRING (UESCAPE STRING)?      #unicodeStringLiteral
    fn parse_string(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TK::String => self.parse_literal(),
            TK::UnicodeString => {
                let string = self.eat_token();
                let uescape_opt = self.eat_opt(TK::UESCAPE);
                let escape = if uescape_opt.is_empty() {
                    self.eat_empty()
                } else {
                    self.eat(TK::String)
                };
                parse_tree::unicode_string(string, uescape_opt, escape)
            }
            _ => self.expected_error("string"),
        }
    }

    fn peek_string(&mut self) -> bool {
        match self.peek() {
            TK::String | TK::UnicodeString => true,
            _ => false,
        }
    }

    // type_
    // : type_ ARRAY
    fn parse_type(&mut self) -> ParseTree<'a> {
        let mut root_type = self.parse_root_type();
        while self.peek_predefined_name(PN::ARRAY) {
            let array = self.eat_predefined_name(PN::ARRAY);
            root_type = parse_tree::array_type_suffix(root_type, array)
        }
        root_type
    }

    fn peek_type_offset(&mut self, offset: usize) -> bool {
        match self.peek_offset(offset) {
            TK::TimeWithTimeZone
            | TK::TimestampWithTimeZone
            | TK::DoublePrecision
            | TK::Identifier => true,
            _ => false,
        }
    }

    // | ARRAY '<' type_ '>'
    // | MAP '<' type_ ',' type_ '>'
    // | ROW '(' identifier type_ (',' identifier type_)* ')'
    // | baseType ('(' typeParameter (',' typeParameter)* ')')?
    // | INTERVAL from_=intervalField TO to=intervalField
    fn parse_root_type(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TK::TimeWithTimeZone | TK::TimestampWithTimeZone | TK::DoublePrecision => (),
            TK::Identifier => {
                match self.maybe_peek_predefined_name() {
                    Some(PN::ARRAY) => {
                        if self.peek_array_type() {
                            return self.parse_array_type();
                        }
                    }
                    Some(PN::MAP) => {
                        if self.peek_map_type() {
                            return self.parse_map_type();
                        }
                    }
                    Some(PN::ROW) => {
                        if self.peek_row_type() {
                            return self.parse_row_type();
                        }
                    }
                    Some(PN::INTERVAL) => {
                        if self.peek_interval_type() {
                            return self.parse_interval_type();
                        }
                    }
                    _ => (),
                };
                ()
            }
            _ => return self.expected_error("type"),
        }
        let type_name = self.eat_token();
        let type_parameters = self
            .parse_parenthesized_comma_separated_list_opt(|parser| parser.parse_type_parameter());
        parse_tree::named_type(type_name, type_parameters)
    }

    // typeParameter
    // : INTEGER_VALUE | type_
    fn parse_type_parameter(&mut self) -> ParseTree<'a> {
        if self.peek_kind(TK::Integer) {
            self.eat_token()
        } else {
            self.parse_type()
        }
    }

    // | ARRAY '<' type_ '>'
    fn peek_array_type(&mut self) -> bool {
        self.peek_predefined_name(PN::ARRAY) && self.peek_kind_offset(TK::OpenAngle, 1)
    }

    fn parse_array_type(&mut self) -> ParseTree<'a> {
        let array = self.eat_predefined_name(PN::ARRAY);
        let (open_angle, element_type, close_angle) =
            self.parse_delimited(TK::OpenAngle, |parser| parser.parse_type(), TK::CloseAngle);
        parse_tree::array_type(array, open_angle, element_type, close_angle)
    }

    // | MAP '<' type_ ',' type_ '>'
    fn peek_map_type(&mut self) -> bool {
        self.peek_predefined_name(PN::MAP) && self.peek_kind_offset(TK::OpenAngle, 1)
    }

    fn parse_map_type(&mut self) -> ParseTree<'a> {
        let map = self.eat_predefined_name(PN::MAP);
        let open_angle = self.eat(TK::OpenAngle);
        let key_type = self.parse_type();
        let comma = self.eat(TK::Comma);
        let value_type = self.parse_type();
        let close_angle = self.eat(TK::CloseAngle);
        parse_tree::map_type(map, open_angle, key_type, comma, value_type, close_angle)
    }

    // | ROW '(' identifier type_ (',' identifier type_)* ')'
    fn peek_row_type(&mut self) -> bool {
        self.peek_predefined_name(PN::ROW)
            && self.peek_kind_offset(TK::OpenParen, 1)
            && self.peek_row_element_offset(2)
    }

    // identifier type_
    fn peek_row_element_offset(&mut self, offset: usize) -> bool {
        self.peek_identifier_offset(offset) && self.peek_type_offset(offset + 1)
    }

    fn parse_row_type(&mut self) -> ParseTree<'a> {
        let row = self.eat_predefined_name(PN::ROW);
        let element_types =
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_row_type_element());
        parse_tree::row_type(row, element_types)
    }

    fn parse_row_type_element(&mut self) -> ParseTree<'a> {
        let identifier = self.parse_identifier();
        let type_ = self.parse_type();
        parse_tree::row_type_element(identifier, type_)
    }

    // | INTERVAL from_=intervalField TO to=intervalField
    fn peek_interval_type(&mut self) -> bool {
        self.peek_predefined_name(PN::INTERVAL) && self.peek_interval_field_offset(1)
    }

    fn parse_interval_type(&mut self) -> ParseTree<'a> {
        let interval = self.eat_predefined_name(PN::INTERVAL);
        let from = self.parse_interval_field();
        let to_kw = self.eat_predefined_name(PN::TO);
        let to = self.parse_interval_field();
        parse_tree::interval_type(interval, from, to_kw, to)
    }

    pub fn parse_statement(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TK::SELECT | TK::TABLE | TK::VALUES | TK::OpenParen | TK::WITH => self.parse_query(),
            TK::CREATE => self.parse_create_statement(),
            TK::INSERT => self.parse_insert_into(),
            TK::DELETE => self.parse_delete(),
            _ => panic!("TODO: Remaining statements"),
        }
    }

    // | CREATE SCHEMA (IF NOT EXISTS)? qualifiedName
    // (WITH properties)?                                             #createSchema
    // | CREATE TABLE (IF NOT EXISTS)? qualifiedName columnAliases?
    //     (COMMENT string)?
    //     (WITH properties)? AS (query | '('query')')
    //     (WITH (NO)? DATA)?                                             #createTableAsSelect
    // | CREATE TABLE (IF NOT EXISTS)? qualifiedName
    //     '(' tableElement (',' tableElement)* ')'
    //      (COMMENT string)?
    //      (WITH properties)?                                            #createTable
    // | CREATE (OR REPLACE)? VIEW qualifiedName AS query                 #createView
    // | CREATE ROLE name=identifier
    //     (WITH ADMIN grantor)?                                          #createRole
    fn parse_create_statement(&mut self) -> ParseTree<'a> {
        match self.peek_offset(1) {
            TK::TABLE => self.parse_create_table(),
            TK::OR => self.parse_create_view(),
            TK::Identifier => match self.maybe_peek_predefined_name_offset(1) {
                Some(PN::SCHEMA) => self.parse_create_schema(),
                Some(PN::VIEW) => self.parse_create_view(),
                Some(PN::ROLE) => self.parse_create_role(),
                _ => self.expected_error("create statement"),
            },
            _ => self.expected_error("create statement"),
        }
    }

    fn parse_create_table(&mut self) -> ParseTree<'a> {
        let create = self.eat(TK::CREATE);
        let table = self.eat(TK::TABLE);
        let if_not_exists_opt = self.parse_if_not_exists_opt();
        let table_name = self.parse_qualified_name();
        if self.peek_kind(TK::OpenParen) && self.peek_table_element_offset(1) {
            // | CREATE TABLE (IF NOT EXISTS)? qualifiedName
            //     '(' tableElement (',' tableElement)* ')'
            //      (COMMENT string)?
            //      (WITH properties)?                                            #createTable
            let table_elements = self
                .parse_parenthesized_comma_separated_list(|parser| parser.parse_table_element());
            let comment_opt = self.parse_comment_opt();
            let with_properties_opt = self.parse_with_properties_opt();
            parse_tree::create_table(
                create,
                table,
                if_not_exists_opt,
                table_name,
                table_elements,
                comment_opt,
                with_properties_opt,
            )
        } else {
            // | CREATE TABLE (IF NOT EXISTS)? qualifiedName columnAliases?
            //     (COMMENT string)?
            //     (WITH properties)? AS (query | '('query')')
            //     (WITH (NO)? DATA)?                                             #createTableAsSelect
            let column_aliases_opt = self.parse_column_aliases_opt();
            let comment_opt = self.parse_comment_opt();
            let with_properties_opt = self.parse_with_properties_opt();
            let as_ = self.eat(TK::AS);
            let (open_paren_opt, query, close_paren_opt) = if self.peek_kind(TK::OpenParen) {
                self.parse_parenthesized_query()
            // TODO: Need to handle (query_no_with) query_primary_tail
            } else {
                (self.eat_empty(), self.parse_query(), self.eat_empty())
            };
            let with_data_opt = self.parse_with_data_opt();
            parse_tree::create_table_as_select(
                create,
                table,
                if_not_exists_opt,
                table_name,
                column_aliases_opt,
                comment_opt,
                with_properties_opt,
                as_,
                open_paren_opt,
                query,
                close_paren_opt,
                with_data_opt,
            )
        }
    }

    // (IF NOT EXISTS)?
    fn parse_if_not_exists_opt(&mut self) -> ParseTree<'a> {
        let if_ = self.eat_predefined_name_opt(PN::IF);
        if if_.is_empty() {
            if_
        } else {
            let not = self.eat(TK::NOT);
            let exists = self.eat(TK::EXISTS);
            parse_tree::if_not_exists(if_, not, exists)
        }
    }

    // (COMMENT string)?
    fn parse_comment_opt(&mut self) -> ParseTree<'a> {
        let comment = self.eat_predefined_name_opt(PN::COMMENT);
        if comment.is_empty() {
            comment
        } else {
            let value = self.parse_string();
            parse_tree::comment(comment, value)
        }
    }

    // principal
    // : USER identifier       #userPrincipal
    // | ROLE identifier       #rolePrincipal
    // | identifier            #unspecifiedPrincipal
    // ;
    fn parse_principal(&mut self) -> ParseTree<'a> {
        let name = self.maybe_peek_predefined_name();
        let is_identifier = self.peek_identifier_offset(1);
        match (is_identifier, name) {
            (true, Some(PN::USER)) => parse_tree::user_principal(
                self.eat_predefined_name(PN::USER),
                self.parse_identifier(),
            ),
            (true, Some(PN::ROLE)) => parse_tree::role_principal(
                self.eat_predefined_name(PN::ROLE),
                self.parse_identifier(),
            ),
            (_, _) => parse_tree::unspecified_principal(self.parse_identifier()),
        }
    }
    // grantor
    // : CURRENT_USER          #currentUserGrantor
    // | CURRENT_ROLE          #currentRoleGrantor
    // | principal             #specifiedPrincipal
    // ;
    fn parse_grantor(&mut self) -> ParseTree<'a> {
        if self.peek_kind(TK::CURRENT_USER) {
            self.eat_token()
        } else if self.peek_predefined_name(PN::CURRENT_ROLE) {
            self.eat_predefined_name(PN::CURRENT_ROLE)
        } else {
            self.parse_principal()
        }
    }

    // (WITH ADMIN grantor)?
    fn parse_with_admin_grantor_opt(&mut self) -> ParseTree<'a> {
        let with = self.eat_opt(TK::WITH);
        if with.is_empty() {
            with
        } else {
            let admin = self.eat_predefined_name(PN::ADMIN);
            let grantor = self.parse_grantor();
            parse_tree::with_admin_grantor(with, admin, grantor)
        }
    }

    // (WITH properties)?
    fn parse_with_properties_opt(&mut self) -> ParseTree<'a> {
        let with = self.eat_opt(TK::WITH);
        if with.is_empty() {
            with
        } else {
            let properties =
                self.parse_parenthesized_comma_separated_list(|parser| parser.parse_property());
            parse_tree::with_properties(with, properties)
        }
    }

    // property_
    // : identifier EQ expression
    fn parse_property(&mut self) -> ParseTree<'a> {
        let identifier = self.parse_identifier();
        let eq = self.eat(TK::Equal);
        let value = self.parse_expression();
        parse_tree::property(identifier, eq, value)
    }

    // (WITH (NO)? DATA)?
    fn parse_with_data_opt(&mut self) -> ParseTree<'a> {
        let with = self.eat_opt(TK::WITH);
        if with.is_empty() {
            with
        } else {
            let no_opt = self.eat_predefined_name_opt(PN::NO);
            let data = self.eat_predefined_name(PN::DATA);
            parse_tree::with_data(with, no_opt, data)
        }
    }

    // tableElement
    //     : columnDefinition
    //     | likeClause
    //     ;
    fn peek_table_element_offset(&mut self, offset: usize) -> bool {
        // disambiguating between tableElement and column_aliases
        self.peek_kind_offset(TK::LIKE, offset)
            || (self.peek_kind_offset(TK::Identifier, offset)
                && !self.peek_kind_offset(TK::Comma, offset + 1))
    }

    fn parse_table_element(&mut self) -> ParseTree<'a> {
        if self.peek_kind(TK::LIKE) {
            self.parse_like_clause()
        } else {
            self.parse_column_definition()
        }
    }

    // columnDefinition
    //     : identifier type_ (NOT NULL)? (COMMENT string)? (WITH properties)?
    fn parse_column_definition(&mut self) -> ParseTree<'a> {
        let identifier = self.parse_identifier();
        let type_ = self.parse_type();
        let not_null_opt = self.parse_not_null_opt();
        let comment_opt = self.parse_comment_opt();
        let with_properties_opt = self.parse_with_properties_opt();
        parse_tree::column_definition(
            identifier,
            type_,
            not_null_opt,
            comment_opt,
            with_properties_opt,
        )
    }

    // (NOT NULL)?
    fn parse_not_null_opt(&mut self) -> ParseTree<'a> {
        let not = self.eat_opt(TK::NOT);
        if not.is_empty() {
            not
        } else {
            let null = self.eat(TK::NULL);
            parse_tree::not_null(not, null)
        }
    }

    // likeClause
    //     : LIKE qualifiedName (optionType=(INCLUDING | EXCLUDING) PROPERTIES)?
    fn parse_like_clause(&mut self) -> ParseTree<'a> {
        let like = self.eat(TK::LIKE);
        let name = self.parse_qualified_name();
        let (option_type_opt, properties) = if match self.maybe_peek_predefined_name() {
            Some(PN::INCLUDING) | Some(PN::EXCLUDING) => true,
            _ => false,
        } {
            (self.eat_token(), self.eat_predefined_name(PN::PROPERTIES))
        } else {
            (self.eat_empty(), self.eat_empty())
        };
        parse_tree::like_clause(like, name, option_type_opt, properties)
    }

    fn parse_create_view(&mut self) -> ParseTree<'a> {
        let create = self.eat(TK::CREATE);

        let (or, replace) = match self.peek_kind(TK::OR) {
            true => (self.eat_token(), self.eat_predefined_name(PN::REPLACE)),
            false => (self.eat_empty(), self.eat_empty()),
        };
        let view = self.eat_predefined_name(PN::VIEW);
        let qualified_name = self.parse_qualified_name();
        let as_ = self.eat(TK::AS);
        let query = self.parse_query();
        parse_tree::create_view(create, or ,replace, view, qualified_name, as_, query)
    }

    fn parse_create_schema(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_create_role(&mut self) -> ParseTree<'a> {
        let create = self.eat(TK::CREATE);
        let role = self.eat_predefined_name(PN::ROLE);
        let name = self.parse_identifier();
        let with_admin_grantor_opt = self.parse_with_admin_grantor_opt();
        parse_tree::create_role(create, role, name, with_admin_grantor_opt)
    }

    // | INSERT INTO qualifiedName columnAliases? query                   #insertInto
    fn parse_insert_into(&mut self) -> ParseTree<'a> {
        let insert = self.eat(TK::INSERT);
        let into = self.eat(TK::INTO);
        let table_name = self.parse_qualified_name();
        let column_aliases_opt = self.parse_column_aliases_opt();
        let query = self.parse_query();
        parse_tree::insert_into(insert, into, table_name, column_aliases_opt, query)
    }

    // | DELETE FROM qualifiedName (WHERE booleanExpression)?             #delete
    fn parse_delete(&mut self) -> ParseTree<'a> {
        let delete = self.eat(TK::DELETE);
        let from = self.eat(TK::FROM);
        let table_name = self.parse_qualified_name();
        let where_opt = self.eat_opt(TK::WHERE);
        let predicate = if where_opt.is_empty() {
            self.eat_empty()
        } else {
            self.parse_boolean_expression()
        };
        parse_tree::delete(delete, from, table_name, where_opt, predicate)
    }
}

fn errors_of_tree<'a>(tree: &'a ParseTree<'a>) -> Vec<&'a SyntaxError> {
    let mut errors: Vec<&'a SyntaxError> = Vec::new();
    let mut visit = |tree: &'a ParseTree<'a>| match tree {
        ParseTree::Token(tree) => {
            for error in &tree.token.errors {
                errors.push(&error)
            }
        }
        ParseTree::Error(error) => errors.push(&error.error),
        _ => (),
    };
    visit_post_order(tree, &mut visit);
    errors
}

type ParseResult<'a> = (ParseTree<'a>, Vec<SyntaxError>);

/// Parses text for the given element.
/// Returns the parse tree and all errors.
fn parse_entrypoint<'a>(text: &'a str, parse_element: ElementParser<'a>) -> ParseResult<'a> {
    let mut parser = Parser::new(text);
    let tree = parser.parse_entrypoint(parse_element);
    let mut errors = Vec::new();
    for error in errors_of_tree(&tree) {
        errors.push(error.clone())
    }
    errors.append(&mut parser.errors);
    errors.sort_by_key(|error| error.get_range());
    (tree, errors)
}

/// Parses text containing a statement.
/// The errors returned includes all errors contained within the tree.
pub fn parse_statement<'a>(text: &'a str) -> ParseResult<'a> {
    parse_entrypoint(text, |parser| parser.parse_statement())
}
