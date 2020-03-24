use crate::lexing::{
    lexer::Lexer, position, position::Position, predefined_names, predefined_names::PredefinedName,
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

    fn maybe_peek_predefined_name_offset(&mut self, offset: usize) -> Option<PredefinedName> {
        let token = self.peek_token_offset(offset);
        if token.kind == TokenKind::Identifier {
            None
        } else {
            predefined_names::maybe_get_predefined_name(token.value)
        }
    }

    fn maybe_peek_predefined_name(&mut self) -> Option<PredefinedName> {
        self.maybe_peek_predefined_name_offset(0)
    }

    fn peek_predefined_name_offset(&mut self, name: PredefinedName, offset: usize) -> bool {
        self.maybe_peek_predefined_name_offset(offset) == Some(name)
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
        if self.peek_identifier() {
            self.eat_token()
        } else {
            self.expected_error_kind(TokenKind::Identifier)
        }
    }

    fn peek_identifier_offset(&mut self, offset: usize) -> bool {
        match self.peek_offset(offset) {
            TokenKind::Identifier
            | TokenKind::QuotedIdentifier
            | TokenKind::BackquotedIdentifier
            | TokenKind::DigitIdentifier => true,
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

    fn peek_query_primary_offset(&mut self, offset: usize) -> bool {
        match self.peek_offset(offset) {
            TokenKind::SELECT | TokenKind::TABLE | TokenKind::VALUES => true,
            TokenKind::OpenParen => self.peek_query_primary_offset(offset + 1),
            _ => false,
        }
    }

    fn peek_query_offset(&mut self, offset: usize) -> bool {
        self.peek_kind_offset(TokenKind::WITH, offset) || self.peek_query_primary_offset(offset)
    }

    // queryPrimary
    // : querySpecification                   #queryPrimaryDefault
    // | TABLE qualifiedName                  #table
    // | VALUES expression (',' expression)*  #inlineTable
    // | '(' queryNoWith  ')'                 #subquery
    fn parse_query_primary(&mut self) -> ParseTree<'a> {
        match self.peek() {
            TokenKind::SELECT => self.parse_query_specification(),
            TokenKind::TABLE => self.parse_table(),
            TokenKind::VALUES => self.parse_inline_table(),
            TokenKind::OpenParen => self.parse_subquery(),
            _ => self.eat(TokenKind::SELECT),
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
        let values = self.eat(TokenKind::VALUES);
        let expressions = self.parse_comma_separated_list(|parser| parser.parse_expression());
        parse_tree::inline_table(values, expressions)
    }

    // | TABLE qualifiedName                  #table
    fn parse_table(&mut self) -> ParseTree<'a> {
        let table = self.eat(TokenKind::TABLE);
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
        let select = self.eat(TokenKind::SELECT);
        let set_quantifier_opt = self.parse_set_quantifier_opt();
        let select_items = self.parse_comma_separated_list(|parser| parser.parse_select_item());
        let from = self.eat_opt(TokenKind::FROM);
        let relations = if from.is_empty() {
            self.eat_empty()
        } else {
            self.parse_comma_separated_list(|parser| parser.parse_relation())
        };
        let where_ = self.eat_opt(TokenKind::WHERE);
        let where_predicate = if where_.is_empty() {
            self.eat_empty()
        } else {
            self.parse_boolean_expression()
        };
        let group = self.eat_opt(TokenKind::GROUP);
        let (by, group_by) = if group.is_empty() {
            (self.eat_empty(), self.eat_empty())
        } else {
            let by = self.eat(TokenKind::BY);
            let group_by = self.parse_group_by();
            (by, group_by)
        };
        let having = self.eat_opt(TokenKind::HAVING);
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
        let asterisk = self.eat_opt(TokenKind::Asterisk);
        if asterisk.is_empty() {
            if self.peek_qualified_select_all() {
                let qualifier = self.parse_qualified_name();
                let period = self.eat(TokenKind::Period);
                let asterisk = self.eat(TokenKind::Asterisk);
                parse_tree::qualified_select_all(qualifier, period, asterisk)
            } else {
                let expression = self.parse_expression();
                let as_ = self.eat_opt(TokenKind::AS);
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
            if self.peek_kind_offset(TokenKind::Period, offset) {
                offset += 1;
            } else {
                return false;
            }
        }
        offset > 0 && self.peek_kind(TokenKind::Asterisk)
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
                TokenKind::CROSS => {
                    let cross = self.eat(TokenKind::CROSS);
                    let join = self.eat(TokenKind::JOIN);
                    let right = self.parse_sampled_relation();
                    left = parse_tree::cross_join(left, cross, join, right)
                }
                TokenKind::INNER | TokenKind::LEFT | TokenKind::RIGHT | TokenKind::FULL => {
                    let join_type = self.parse_join_type();
                    let join = self.eat(TokenKind::JOIN);
                    let right = self.parse_relation();
                    let join_criteria = self.parse_join_criteria();
                    left = parse_tree::join(left, join_type, join, right, join_criteria)
                }
                TokenKind::NATURAL => {
                    let natural = self.eat(TokenKind::CROSS);
                    let join_type = self.parse_join_type();
                    let join = self.eat(TokenKind::JOIN);
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
            TokenKind::INNER => self.eat(TokenKind::INNER),
            TokenKind::LEFT | TokenKind::RIGHT | TokenKind::FULL => {
                let kind = self.eat_token();
                let outer_opt = self.eat_opt(TokenKind::OUTER);
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
            TokenKind::ON => {
                let on = self.eat(TokenKind::ON);
                let predicate = self.parse_boolean_expression();
                parse_tree::on_join_criteria(on, predicate)
            }
            TokenKind::USING => {
                let using = self.eat(TokenKind::USING);
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
        let tablesample = self.eat_predefined_name_opt(PredefinedName::TABLESAMPLE);
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
        let bernoulli = self.eat_predefined_name_opt(PredefinedName::BERNOULLI);
        if bernoulli.is_empty() {
            self.eat_predefined_name(PredefinedName::SYSTEM)
        } else {
            bernoulli
        }
    }

    // aliasedRelation
    // : relationPrimary (AS? identifier columnAliases?)?
    fn parse_aliased_relation(&mut self) -> ParseTree<'a> {
        let relation_primary = self.parse_relation_primary();
        if self.peek_kind(TokenKind::AS) || self.peek_identifier() {
            let as_opt = self.eat_opt(TokenKind::AS);
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
            TokenKind::OpenParen => {
                if self.peek_query_offset(1) {
                    let (open_paren, query, close_paren) =
                        self.parse_parenthesized(|parser| parser.parse_query());
                    parse_tree::subquery_relation(open_paren, query, close_paren)
                } else {
                    let (open_paren, relation, close_paren) =
                        self.parse_parenthesized(|parser| parser.parse_relation());
                    parse_tree::parenthesized_relation(open_paren, relation, close_paren)
                }
            }
            TokenKind::UNNEST => self.parse_unnest(),
            _ => {
                if self.peek_predefined_name(PredefinedName::LATERAL)
                    && self.peek_kind_offset(TokenKind::OpenParen, 1)
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
        let lateral = self.eat_predefined_name(PredefinedName::LATERAL);
        let (open_paren, query, close_paren) =
            self.parse_parenthesized(|parser| parser.parse_query());
        parse_tree::lateral(lateral, open_paren, query, close_paren)
    }

    // | UNNEST '(' expression (',' expression)* ')' (WITH ORDINALITY)?  #unnest
    fn parse_unnest(&mut self) -> ParseTree<'a> {
        let unnest = self.eat(TokenKind::UNNEST);
        let expressions =
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_expression());
        let with = self.eat_opt(TokenKind::WITH);
        let ordinality = if with.is_empty() {
            self.eat_empty()
        } else {
            self.eat_predefined_name(PredefinedName::ORDINALITY)
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
            TokenKind::ROLLUP => self.parse_rollup(),
            TokenKind::CUBE => self.parse_cube(),
            TokenKind::GROUPING => self.parse_grouping_sets(),
            _ => self.parse_grouping_set(),
        }
    }

    // | ROLLUP '(' (expression (',' expression)*)? ')'         #rollup
    fn parse_rollup(&mut self) -> ParseTree<'a> {
        let rollup = self.eat(TokenKind::ROLLUP);
        let expressions =
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_expression());
        parse_tree::rollup(rollup, expressions)
    }

    // | CUBE '(' (expression (',' expression)*)? ')'           #cube
    fn parse_cube(&mut self) -> ParseTree<'a> {
        let cube = self.eat(TokenKind::CUBE);
        let expressions =
            self.parse_parenthesized_comma_separated_list(|parser| parser.parse_expression());
        parse_tree::cube(cube, expressions)
    }

    // | GROUPING SETS '(' groupingSet (',' groupingSet)* ')'   #multipleGroupingSets
    fn parse_grouping_sets(&mut self) -> ParseTree<'a> {
        let grouping = self.eat(TokenKind::GROUPING);
        let sets = self.eat_predefined_name(PredefinedName::SETS);
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
            |parser| parser.peek_kind(TokenKind::OR),
            |parser| parser.parse_and_expression(),
        )
    }

    // | left=booleanExpression operator=AND right=booleanExpression  #logicalBinary
    fn parse_and_expression(&mut self) -> ParseTree<'a> {
        self.parse_binary_expression(
            |parser| parser.peek_kind(TokenKind::AND),
            |parser| parser.parse_not_expression(),
        )
    }

    // | NOT booleanExpression                                        #logicalNot
    fn parse_not_expression(&mut self) -> ParseTree<'a> {
        let not = self.eat_opt(TokenKind::NOT);
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
            let (open_paren, query, close_paren) =
                self.parse_parenthesized(|parser| parser.parse_query());
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
            TokenKind::Equal
            | TokenKind::LessGreater
            | TokenKind::BangEqual
            | TokenKind::OpenAngle
            | TokenKind::CloseAngle
            | TokenKind::LessEqual
            | TokenKind::GreaterEqual => true,
            _ => false,
        }
    }

    // | IS NOT? NULL                                                        #nullPredicate
    // | IS NOT? DISTINCT FROM right=valueExpression                         #distinctFrom
    fn parse_is_suffix(&mut self, value: ParseTree<'a>) -> ParseTree<'a> {
        assert!(self.peek_kind(TokenKind::IS));
        let is = self.eat_token();
        let not_opt = self.eat_opt(TokenKind::NOT);
        match self.peek() {
            TokenKind::NULL => {
                let null = self.eat_token();
                parse_tree::null_predicate(value, is, not_opt, null)
            }
            TokenKind::DISTINCT => {
                let distinct = self.eat_token();
                let from = self.eat(TokenKind::FROM);
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
        let between = self.eat(TokenKind::BETWEEN);
        let lower = self.parse_value_expression();
        let and = self.eat(TokenKind::AND);
        let upper = self.parse_value_expression();
        parse_tree::between(value, not_opt, between, lower, and, upper)
    }

    // | NOT? LIKE pattern=valueExpression (ESCAPE escape=valueExpression)?  #like
    fn parse_like_suffix(&mut self, value: ParseTree<'a>, not_opt: ParseTree<'a>) -> ParseTree<'a> {
        let like = self.eat(TokenKind::LIKE);
        let pattern = self.parse_value_expression();
        let escape_opt = self.eat_opt(TokenKind::ESCAPE);
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
        let in_ = self.eat(TokenKind::IN);
        if self.peek_kind(TokenKind::OpenParen) && self.peek_query_primary_offset(1) {
            let (open_paren, query, close_paren) =
                self.parse_parenthesized(|parser| parser.parse_query());
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
            TokenKind::Equal
            | TokenKind::LessGreater
            | TokenKind::BangEqual
            | TokenKind::OpenAngle
            | TokenKind::CloseAngle
            | TokenKind::LessEqual
            | TokenKind::GreaterEqual => self.parse_comparison_operator_suffix(value),
            TokenKind::IS => self.parse_is_suffix(value),
            _ => {
                let not_opt = self.eat_opt(TokenKind::NOT);
                match self.peek() {
                    TokenKind::BETWEEN => self.parse_between_suffix(value, not_opt),
                    TokenKind::IN => self.parse_in_suffix(value, not_opt),
                    TokenKind::LIKE => self.parse_like_suffix(value, not_opt),
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
            Some(PredefinedName::ALL) | Some(PredefinedName::SOME) | Some(PredefinedName::ANY) => {
                true
            }
            _ => false,
        }
    }

    fn peek_quantified_comparison(&mut self) -> bool {
        self.peek_comparison_quantifier()
            && self.peek_kind_offset(TokenKind::OpenParen, 1)
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
            |parser| parser.peek_kind(TokenKind::BarBar),
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
            TokenKind::Plus | TokenKind::Minus => true,
            _ => false,
        }
    }

    // | left=valueExpression operator=(ASTERISK | SLASH | PERCENT) right=valueExpression  #arithmeticBinary
    fn parse_multiplicative_expression(&mut self) -> ParseTree<'a> {
        self.parse_binary_expression(
            |parser| match parser.peek() {
                TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percent => true,
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
        let at = self.eat_predefined_name_opt(PredefinedName::AT);
        if at.is_empty() {
            value
        } else {
            let time = self.eat_predefined_name(PredefinedName::TIME);
            let zone = self.eat_predefined_name(PredefinedName::ZONE);
            let specifier = if self.peek_predefined_name(PredefinedName::INTERVAL) {
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
            self.parse_separated_list(TokenKind::Period, |parser| parser.parse_identifier()),
        )
    }

    fn parse_primary_prefix_expression(&mut self) -> ParseTree<'a> {
        match self.peek() {
            // : NULL                                                                                #nullLiteral
            TokenKind::NULL => self.parse_literal(),
            // | DOUBLE_PRECISION string                                                             #typeConstructor
            TokenKind::DoublePrecision => self.parse_type_constructor(),
            // | booleanValue                                                                        #booleanLiteral
            TokenKind::TRUE | TokenKind::FALSE => self.parse_literal(),
            // | number                                                                              #numericLiteral
            TokenKind::Decimal | TokenKind::Double | TokenKind::Integer => self.parse_literal(),
            // | string                                                                              #stringLiteral
            TokenKind::String | TokenKind::UnicodeString => self.parse_literal(),
            // | BINARY_LITERAL                                                                      #binaryLiteral
            TokenKind::BinaryLiteral => self.parse_literal(),
            // | '?'                                                                                 #parameter
            TokenKind::Question => self.parse_parameter(),
            // // This is an extension to ANSI SQL, which considers EXISTS to be a <boolean expression>
            // | EXISTS '(' query ')'                                                                #exists
            TokenKind::EXISTS => self.parse_exists(),
            // | CASE valueExpression whenClause+ (ELSE elseExpression=expression)? END              #simpleCase
            // | CASE whenClause+ (ELSE elseExpression=expression)? END                              #searchedCase
            TokenKind::CASE => self.parse_case(),
            // | CAST '(' expression AS type_ ')'                                                     #cast
            TokenKind::CAST => self.parse_cast(),
            // | name=CURRENT_DATE                                                                   #specialDateTimeFunction
            TokenKind::CURRENT_DATE => self.parse_current_date(),
            // | name=CURRENT_TIME ('(' precision=INTEGER_VALUE ')')?                                #specialDateTimeFunction
            TokenKind::CURRENT_TIME => self.parse_current_time(),
            // | name=CURRENT_TIMESTAMP ('(' precision=INTEGER_VALUE ')')?                           #specialDateTimeFunction
            TokenKind::CURRENT_TIMESTAMP => self.parse_current_timestamp(),
            // | name=LOCALTIME ('(' precision=INTEGER_VALUE ')')?                                   #specialDateTimeFunction
            TokenKind::LOCALTIME => self.parse_localtime(),
            // | name=LOCALTIMESTAMP ('(' precision=INTEGER_VALUE ')')?                              #specialDateTimeFunction
            TokenKind::LOCALTIMESTAMP => self.parse_localtimestamp(),
            // | name=CURRENT_USER                                                                   #currentUser
            TokenKind::CURRENT_USER => self.parse_current_user(),
            // | name=CURRENT_PATH                                                                   #currentPath
            TokenKind::CURRENT_PATH => self.parse_current_path(),
            // | NORMALIZE '(' valueExpression (',' normalForm)? ')'                                 #normalize
            TokenKind::NORMALIZE => self.parse_normalize(),
            // | EXTRACT '(' identifier FROM valueExpression ')'                                     #extract
            TokenKind::EXTRACT => self.parse_extract(),
            // | GROUPING '(' (qualifiedName (',' qualifiedName)*)? ')'                              #groupingOperation
            TokenKind::GROUPING => self.parse_grouping(),
            // | configureExpression                                                                 #conf
            TokenKind::CONFIGURE => self.parse_configure_expression(),

            // | '(' expression (',' expression)+ ')'                                                #rowConstructor
            // | '(' (identifier (',' identifier)*)? ')' '->' expression                             #lambda
            // | '(' query ')'                                                                       #subqueryExpression
            // | '(' expression ')'                                                                  #parenthesizedExpression
            TokenKind::OpenParen => {
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
            TokenKind::Identifier => {
                if let Some(name) = self.maybe_peek_predefined_name() {
                    match name {
                        PredefinedName::INTERVAL => {
                            if self.peek_interval() {
                                return self.parse_interval();
                            }
                        }
                        PredefinedName::POSITION => {
                            if self.peek_position() {
                                return self.parse_position();
                            }
                        }
                        PredefinedName::ROW => {
                            if self.peek_row_constructor() {
                                return self.parse_row_constructor();
                            }
                        }
                        PredefinedName::TRY_CAST => {
                            if self.peek_try_cast() {
                                return self.parse_try_cast();
                            }
                        }
                        PredefinedName::ARRAY => {
                            if self.peek_array_constructor() {
                                return self.parse_array_constructor();
                            }
                        }
                        PredefinedName::SUBSTRING => {
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
            TokenKind::QuotedIdentifier
            | TokenKind::BackquotedIdentifier
            | TokenKind::DigitIdentifier => self.parse_identifier_start_expression(),
            _ => self.expected_error("Expected expression."),
        }
    }

    fn parse_primary_expression(&mut self) -> ParseTree<'a> {
        let mut result = self.parse_primary_prefix_expression();
        loop {
            // suffixes
            match self.peek() {
                // | base=primaryExpression '.' fieldName=identifier                                     #dereference
                TokenKind::Period => {
                    let period = self.eat(TokenKind::Period);
                    let field_name = self.parse_identifier();
                    result = parse_tree::dereference(result, period, field_name)
                }
                // | value=primaryExpression '[' index=valueExpression ']'                               #subscript
                TokenKind::OpenSquare => {
                    let open_square = self.eat(TokenKind::OpenSquare);
                    let index = self.parse_value_expression();
                    let close_square = self.eat(TokenKind::CloseSquare);
                    result = parse_tree::subscript(result, open_square, index, close_square)
                }
                _ => return result,
            }
        }
    }

    fn peek_lambda(&mut self) -> bool {
        panic!("TODO")
    }

    fn parse_lambda(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_row_constructor_or_paren_expression(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn peek_position(&mut self) -> bool {
        panic!("TODO")
    }

    fn parse_position(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn peek_interval(&mut self) -> bool {
        panic!("TODO")
    }

    fn parse_interval(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn peek_row_constructor(&mut self) -> bool {
        panic!("TODO")
    }

    fn parse_row_constructor(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn peek_try_cast(&mut self) -> bool {
        panic!("TODO")
    }

    fn parse_try_cast(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn peek_array_constructor(&mut self) -> bool {
        panic!("TODO")
    }

    fn parse_array_constructor(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn peek_configure_expression(&mut self) -> bool {
        panic!("TODO")
    }

    fn parse_configure_expression(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn peek_substring(&mut self) -> bool {
        panic!("TODO")
    }

    fn parse_substring(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_subquery_expression(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_grouping(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_extract(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_current_path(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_current_user(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_current_date(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_current_time(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_current_timestamp(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_normalize(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_localtimestamp(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_localtime(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_cast(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_case(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_exists(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_parameter(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_literal(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_type_constructor(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_subscript(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_dereference(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    // | identifier string                                                                   #typeConstructor
    // | qualifiedName '(' ASTERISK ')' filter_? over?                                        #functionCall
    // | qualifiedName '(' (setQuantifier? expression (',' expression)*)?
    //     (ORDER BY sortItem (',' sortItem)*)? ')' filter_? over?                            #functionCall
    // | identifier '->' expression                                                          #lambda
    // | identifier                                                                          #columnReference
    fn parse_identifier_start_expression(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_string(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }

    fn parse_statement(&mut self) -> ParseTree<'a> {
        panic!("TODO")
    }
}
