use crate::lexing::{
    lexer::Lexer, position, position::Position, predefined_names,
    predefined_names::PredefinedName as PN, text_range::TextRange, token::Token,
    token_kind::TokenKind as TK,
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

    pub fn peek_offset(&mut self, offset: usize) -> TK {
        self.peek_token_offset(offset).kind
    }

    pub fn peek_kind_offset(&mut self, kind: TK, offset: usize) -> bool {
        self.peek_offset(offset) == kind
    }

    pub fn peek_kind(&mut self, kind: TK) -> bool {
        self.peek_kind_offset(kind, 0)
    }

    pub fn peek(&mut self) -> TK {
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
type Peeker<'a> = fn(&mut Parser<'a>) -> bool;

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

    fn maybe_peek_predefined_name_offset(&mut self, offset: usize) -> Option<PN> {
        let token = self.peek_token_offset(offset);
        if token.kind == TK::Identifier {
            None
        } else {
            predefined_names::maybe_get_predefined_name(token.value)
        }
    }

    fn maybe_peek_predefined_name(&mut self) -> Option<PN> {
        self.maybe_peek_predefined_name_offset(0)
    }

    fn peek_predefined_name_offset(&mut self, name: PN, offset: usize) -> bool {
        self.maybe_peek_predefined_name_offset(offset) == Some(name)
    }

    fn peek_predefined_name(&mut self, name: PN) -> bool {
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

    fn expected_error_kind(&mut self, expected: TK) -> ParseTree<'a> {
        self.expected_error(expected.to_string().as_str())
    }

    fn expected_error_name(&mut self, expected: PN) -> ParseTree<'a> {
        self.expected_error(expected.to_string().as_str())
    }

    fn eat(&mut self, kind: TK) -> ParseTree<'a> {
        if self.peek_kind(kind) {
            self.eat_token()
        } else {
            self.expected_error_kind(kind)
        }
    }

    fn eat_predefined_name(&mut self, name: PN) -> ParseTree<'a> {
        if self.peek_predefined_name(name) {
            self.eat_token()
        } else {
            self.expected_error_name(name)
        }
    }

    fn eat_predefined_name_opt(&mut self, name: PN) -> ParseTree<'a> {
        if self.peek_predefined_name(name) {
            self.eat_token()
        } else {
            self.eat_empty()
        }
    }

    fn eat_opt(&mut self, kind: TK) -> ParseTree<'a> {
        if self.peek_kind(kind) {
            self.eat_token()
        } else {
            self.eat_empty()
        }
    }

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

    fn parse_parenthesized(
        &mut self,
        parse_element: ElementParser<'a>,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        self.parse_delimited(TK::OpenParen, parse_element, TK::CloseParen)
    }

    // parse non-empty separated list
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
        } {}
        elements.into_iter().zip(seperators.into_iter()).collect()
    }

    // parse non-empty  list
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

    // parse possibly empty separated list
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

    // Parse non-empty list.
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

    // Parse non-empty separated list.
    // Terminating separator is not consumed.
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

    // Parse possibly-empty separated list.
    // Terminating separator is not consumed.
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

    // Parse non-empty comma separated list.
    // Terminating commas are not consumed.
    fn parse_comma_separated_list(&mut self, parse_element: ElementParser<'a>) -> ParseTree<'a> {
        self.parse_separated_list(TK::Comma, parse_element)
    }

    // Parse possibly-empty comma separated list.
    // Terminating commas are not consumed.
    fn parse_comma_separated_list_opt(
        &mut self,
        peek_element: Peeker<'a>,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        self.parse_separated_list_opt(TK::Comma, peek_element, parse_element)
    }

    // Parse delimited non-empty separated list.
    // Terminating separator is not permitted.
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

    // Parse delimited possibly-empty separated list.
    // Terminating separator is not permitted.
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

    // Parse parenthesized, non-empty comma separated list.
    // Terminating commas are not consumed.
    fn parse_parenthesized_comma_separated_list(
        &mut self,
        parse_element: ElementParser<'a>,
    ) -> ParseTree<'a> {
        self.parse_delimited_separated_list(TK::OpenParen, TK::Comma, parse_element, TK::CloseParen)
    }

    // Parse optional parenthesized, non-empty comma separated list.
    // Terminating commas are not consumed.
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
            op_kind == TK::UNION || op_kind == TK::EXCEPT
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
        while self.peek_kind(TK::INTERSECT) {
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
        let distinct = self.eat_opt(TK::DISTINCT);
        if distinct.is_empty() {
            self.eat_predefined_name_opt(PN::ALL)
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
        let set_quantifier_opt = self.parse_set_quantifier_opt();
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
        offset > 0 && self.peek_kind(TK::Asterisk)
    }

    // relation
    // : left=relation
    //   ( CROSS JOIN right=sampledRelation
    //   | joinType JOIN rightRelation=relation joinCriteria
    //   | NATURAL joinType JOIN right=sampledRelation
    //   )                                           #joinRelation
    // | sampledRelation                             #relationDefault
    fn parse_relation(&mut self) -> ParseTree<'a> {
        let mut left = self.parse_sampled_relation();
        loop {
            match self.peek() {
                TK::CROSS => {
                    let cross = self.eat(TK::CROSS);
                    let join = self.eat(TK::JOIN);
                    let right = self.parse_sampled_relation();
                    left = parse_tree::cross_join(left, cross, join, right)
                }
                TK::INNER | TK::LEFT | TK::RIGHT | TK::FULL => {
                    let join_type = self.parse_join_type();
                    let join = self.eat(TK::JOIN);
                    let right = self.parse_relation();
                    let join_criteria = self.parse_join_criteria();
                    left = parse_tree::join(left, join_type, join, right, join_criteria)
                }
                TK::NATURAL => {
                    let natural = self.eat(TK::CROSS);
                    let join_type = self.parse_join_type();
                    let join = self.eat(TK::JOIN);
                    let right = self.parse_sampled_relation();
                    left = parse_tree::natural_join(left, natural, join_type, join, right)
                }
                _ => return left,
            }
        }
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
        let aliased_relation = self.parse_aliased_relation();
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

    // sampleType
    // : BERNOULLI
    // | SYSTEM
    fn parse_sample_type(&mut self) -> ParseTree<'a> {
        let bernoulli = self.eat_predefined_name_opt(PN::BERNOULLI);
        if bernoulli.is_empty() {
            self.eat_predefined_name(PN::SYSTEM)
        } else {
            bernoulli
        }
    }

    // aliasedRelation
    // : relationPrimary (AS? identifier columnAliases?)?
    fn parse_aliased_relation(&mut self) -> ParseTree<'a> {
        let relation_primary = self.parse_relation_primary();
        if self.peek_kind(TK::AS) || self.peek_identifier() {
            let as_opt = self.eat_opt(TK::AS);
            let identifier = self.parse_identifier();
            let column_aliases_opt = self.parse_column_aliases_opt();
            parse_tree::aliased_relation(relation_primary, as_opt, identifier, column_aliases_opt)
        } else {
            relation_primary
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
                if self.peek_query_offset(1) {
                    let (open_paren, query, close_paren) = self.parse_parenthesized_query();
                    parse_tree::subquery_relation(open_paren, query, close_paren)
                } else {
                    let (open_paren, relation, close_paren) =
                        self.parse_parenthesized(|parser| parser.parse_relation());
                    parse_tree::parenthesized_relation(open_paren, relation, close_paren)
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
        let set_quantifier_opt = self.parse_set_quantifier_opt();
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
        // parenthesized expressions will show up as
        // either a row constructor or a paren expression.
        self.parse_expression()
    }

    // expression
    // : booleanExpression
    fn parse_expression(&mut self) -> ParseTree<'a> {
        self.parse_boolean_expression()
    }

    fn peek_expression(&mut self) -> bool {
        // TODO: tighten this up
        !self.peek_kind(TK::CloseParen)
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
        let mut left = parse_operand(self);
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

    // | left=booleanExpression operator=AND right=booleanExpression  #logicalBinary
    fn parse_and_expression(&mut self) -> ParseTree<'a> {
        self.parse_binary_expression(
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
        assert!(self.peek_comparison_operator());
        let operator = self.eat_token();
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
        assert!(self.peek_kind(TK::IS));
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
        if self.peek_kind(TK::OpenParen) && self.peek_query_primary_offset(1) {
            let (open_paren, query, close_paren) = self.parse_parenthesized_query();
            parse_tree::in_subquery(value, not_opt, in_, open_paren, query, close_paren)
        } else {
            let expressions =
                self.parse_parenthesized_comma_separated_list(|parser| parser.parse_expression());
            parse_tree::in_list(value, not_opt, in_, expressions)
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
        match self.peek() {
            TK::Equal
            | TK::LessGreater
            | TK::BangEqual
            | TK::OpenAngle
            | TK::CloseAngle
            | TK::LessEqual
            | TK::GreaterEqual => self.parse_comparison_operator_suffix(value),
            TK::IS => self.parse_is_suffix(value),
            _ => {
                let not_opt = self.eat_opt(TK::NOT);
                match self.peek() {
                    TK::BETWEEN => self.parse_between_suffix(value, not_opt),
                    TK::IN => self.parse_in_suffix(value, not_opt),
                    TK::LIKE => self.parse_like_suffix(value, not_opt),
                    _ => {
                        if not_opt.is_empty() {
                            value
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

    // | left=valueExpression operator=(PLUS | MINUS) right=valueExpression                #arithmeticBinary
    fn parse_additive_expression(&mut self) -> ParseTree<'a> {
        self.parse_binary_expression(
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
            |parser| match parser.peek() {
                TK::Asterisk | TK::Slash | TK::Percent => true,
                _ => false,
            },
            |parser| parser.parse_arithmetic_unary_expression(),
        )
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
        let value = self.parse_primary_expression();
        let at = self.eat_predefined_name_opt(PN::AT);
        if at.is_empty() {
            value
        } else {
            let time = self.eat_predefined_name(PN::TIME);
            let zone = self.eat_predefined_name(PN::ZONE);
            let specifier = if self.peek_predefined_name(PN::INTERVAL) {
                self.parse_interval()
            } else {
                self.parse_string()
            };
            parse_tree::at_time_zone(value, at, time, zone, specifier)
        }
    }

    // qualifiedName
    // : identifier ('.' identifier)*
    fn parse_qualified_name(&mut self) -> ParseTree<'a> {
        parse_tree::qualified_name(
            self.parse_separated_list(TK::Period, |parser| parser.parse_identifier()),
        )
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
                if self.peek_query_offset(1) {
                    self.parse_subquery_expression()
                } else if self.peek_lambda() {
                    self.parse_lambda()
                } else {
                    self.parse_row_constructor_or_paren_expression()
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
        let mut result = self.parse_primary_prefix_expression();
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

    // | '(' expression (',' expression)+ ')'                                                #rowConstructor
    // | '(' expression ')'                                                                  #parenthesizedExpression
    fn parse_row_constructor_or_paren_expression(&mut self) -> ParseTree<'a> {
        let list =
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_expression());
        if list.as_list().len() == 1 {
            let (start_delimiter, mut elements_and_separators, end_delimiter) = list.unbox_list();
            parse_tree::parenthesized_expression(
                start_delimiter,
                elements_and_separators.remove(0).0,
                end_delimiter,
            )
        } else {
            parse_tree::row_constructor(list)
        }
    }

    fn peek_position(&mut self) -> bool {
        panic!("TODO")
    }

    fn parse_position(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    // interval
    // : INTERVAL sign=(PLUS | MINUS)? (string | configureExpression) from_=intervalField (TO to=intervalField)?
    fn peek_interval(&mut self) -> bool {
        self.peek_predefined_name(PN::INTERVAL)
            && match self.peek_offset(1) {
                TK::Plus | TK::Minus | TK::String | TK::UnicodeString | TK::Identifier => true,
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
    fn parse_interval_field(&mut self) -> ParseTree<'a> {
        match self.maybe_peek_predefined_name() {
            Some(PN::YEAR) | Some(PN::MONTH) | Some(PN::DAY) | Some(PN::HOUR)
            | Some(PN::MINUTE) | Some(PN::SECOND) => self.eat_token(),
            _ => self.expected_error("interval field"),
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
        let localtimestamp = self.eat(TK::CURRENT_TIME);
        let (open_paren, precision, close_paren) = if self.peek_kind(TK::OpenParen) {
            self.parse_parenthesized(|parser| parser.eat(TK::Integer))
        } else {
            (self.eat_empty(), self.eat_empty(), self.eat_empty())
        };
        parse_tree::localtimestamp(localtimestamp, open_paren, precision, close_paren)
    }

    // | name=LOCALTIME ('(' precision=INTEGER_VALUE ')')?                                   #specialDateTimeFunction
    fn parse_localtime(&mut self) -> ParseTree<'a> {
        let localtime = self.eat(TK::CURRENT_TIME);
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

    fn peek_function_call(&mut self) -> bool {
        let mut offset = 1;
        while self.peek_kind(TK::Period) {
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
        let (set_quantifier_opt, arguments, order_by_opt) = if self.peek_kind(TK::Asterisk) {
            (self.eat_empty(), self.eat(TK::Asterisk), self.eat_empty())
        } else {
            let set_quantifier_opt = self.parse_set_quantifier_opt();
            let arguments = if set_quantifier_opt.is_empty() {
                self.parse_comma_separated_list_opt(
                    |parser| parser.peek_expression(),
                    |parser| parser.parse_expression(),
                )
            } else {
                self.parse_comma_separated_list(|parser| parser.parse_expression())
            };
            let order_by_opt = self.parse_order_by_opt();
            (set_quantifier_opt, arguments, order_by_opt)
        };
        let close_paren = self.eat(TK::CloseParen);
        let filter_opt = self.parse_filter_opt();
        let over_opt = self.parse_over_opt();
        parse_tree::function_call(
            name,
            open_paren,
            set_quantifier_opt,
            arguments,
            order_by_opt,
            close_paren,
            filter_opt,
            over_opt,
        )
    }

    fn parse_filter_opt(&mut self) -> ParseTree<'a> {
        let filter = self.eat_predefined_name_opt(PN::FILTER);
        if filter.is_empty() {
            filter
        } else {
            let open_paren = self.eat(TK::OpenParen);
            let where_ = self.eat(TK::WHERE);
            let predicate = self.parse_boolean_expression();
            let close_paren = self.eat(TK::CloseParen);
            parse_tree::filter(filter, open_paren, where_, predicate, close_paren)
        }
    }

    // over
    // : OVER '('
    //     (PARTITION BY partition+=expression (',' partition+=expression)*)?
    //     (ORDER BY sortItem (',' sortItem)*)?
    //     windowFrame?
    //   ')'
    fn parse_over_opt(&mut self) -> ParseTree<'a> {
        let over = self.eat_predefined_name_opt(PN::OVER);
        if over.is_empty() {
            over
        } else {
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
            let window_frame = self.parse_window_frame();
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
    }

    // windowFrame
    // : frameType=RANGE startBound=frameBound
    // | frameType=ROWS startBound=frameBound
    // | frameType=RANGE BETWEEN startBound=frameBound AND end=frameBound
    // | frameType=ROWS BETWEEN startBound=frameBound AND end=frameBound
    fn parse_window_frame(&mut self) -> ParseTree<'a> {
        let frame_type =
            if self.peek_predefined_name(PN::RANGE) || self.peek_predefined_name(PN::ROWS) {
                self.eat_token()
            } else {
                self.expected_error("RANGE, ROWS")
            };
        let between_opt = self.eat_opt(TK::BETWEEN);
        let start = self.parse_frame_bound();
        let (and, end) = if between_opt.is_empty() {
            (self.eat_empty(), self.eat_empty())
        } else {
            (self.eat(TK::AND), self.parse_frame_bound())
        };
        parse_tree::window_frame(frame_type, between_opt, start, and, end)
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

    fn parse_type(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_statement(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }
}
