use crate::lexing::{text_range::TextRange, token};

#[derive(Clone, Debug)]
pub enum ParseTree<'a> {
    // The core trees
    Empty(Empty),
    Token(Token<'a>),
    List(List<'a>),
    Error(Error),

    // The language specific trees
    Query(Query<'a>),
    With(With<'a>),
    NamedQuery(NamedQuery<'a>),
    QueryNoWith(QueryNoWith<'a>),
    OrderBy(OrderBy<'a>),
    Limit(Limit<'a>),
    QuerySetOperation(QuerySetOperation<'a>),
}

// The core trees
#[derive(Clone, Debug)]
pub struct Empty {
    pub range: TextRange,
}

pub fn empty<'a>(range: TextRange) -> ParseTree<'a> {
    ParseTree::Empty(Empty { range })
}

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub token: token::Token<'a>,
}

pub fn token<'a>(token: token::Token<'a>) -> ParseTree<'a> {
    ParseTree::Token(Token { token })
}

#[derive(Clone, Debug)]
pub struct List<'a> {
    pub start_delimiter: Box<ParseTree<'a>>,
    pub elements_and_separators: Vec<(ParseTree<'a>, ParseTree<'a>)>,
    pub end_delimiter: Box<ParseTree<'a>>,
}

pub fn list<'a>(
    start_delimiter: ParseTree<'a>,
    elements_and_separators: Vec<(ParseTree<'a>, ParseTree<'a>)>,
    end_delimiter: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::List(List {
        start_delimiter: Box::new(start_delimiter),
        elements_and_separators,
        end_delimiter: Box::new(end_delimiter),
    })
}

#[derive(Clone, Debug)]
pub struct Error {
    pub range: TextRange,
    pub message: String,
}

pub fn error<'a>(range: TextRange, message: String) -> ParseTree<'a> {
    ParseTree::Error(Error {
        range,
        message: message,
    })
}

// core impl
impl<'a> ParseTree<'a> {
    pub fn is_empty(&self) -> bool {
        if let ParseTree::Empty(_) = self {
            true
        } else {
            false
        }
    }
}

// The language specific trees
#[derive(Clone, Debug)]
pub struct Query<'a> {
    pub with: Box<ParseTree<'a>>,
    pub query_no_with: Box<ParseTree<'a>>,
}

pub fn query<'a>(with: ParseTree<'a>, query_no_with: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Query(Query {
        with: Box::new(with),
        query_no_with: Box::new(query_no_with),
    })
}

#[derive(Clone, Debug)]
pub struct With<'a> {
    pub with: Box<ParseTree<'a>>,
    pub recursive: Box<ParseTree<'a>>,
    pub named_queries: Box<ParseTree<'a>>,
}

pub fn with<'a>(
    with: ParseTree<'a>,
    recursive: ParseTree<'a>,
    named_queries: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::With(With {
        with: Box::new(with),
        recursive: Box::new(recursive),
        named_queries: Box::new(named_queries),
    })
}

#[derive(Clone, Debug)]
pub struct NamedQuery<'a> {
    pub name: Box<ParseTree<'a>>,
    pub column_aliases_opt: Box<ParseTree<'a>>,
    pub as_: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn named_query<'a>(
    name: ParseTree<'a>,
    column_aliases_opt: ParseTree<'a>,
    as_: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    query: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::NamedQuery(NamedQuery {
        name: Box::new(name),
        column_aliases_opt: Box::new(column_aliases_opt),
        as_: Box::new(as_),
        open_paren: Box::new(open_paren),
        query: Box::new(query),
        close_paren: Box::new(close_paren),
    })
}

#[derive(Clone, Debug)]
pub struct QueryNoWith<'a> {
    pub query_term: Box<ParseTree<'a>>,
    pub order_by_opt: Box<ParseTree<'a>>,
    pub limit_opt: Box<ParseTree<'a>>,
}

pub fn query_no_with<'a>(
    query_term: ParseTree<'a>,
    order_by_opt: ParseTree<'a>,
    limit_opt: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::QueryNoWith(QueryNoWith {
        query_term: Box::new(query_term),
        order_by_opt: Box::new(order_by_opt),
        limit_opt: Box::new(limit_opt),
    })
}

#[derive(Clone, Debug)]
pub struct OrderBy<'a> {
    pub order: Box<ParseTree<'a>>,
    pub by: Box<ParseTree<'a>>,
    pub sort_items: Box<ParseTree<'a>>,
}

pub fn order_by<'a>(
    order: ParseTree<'a>,
    by: ParseTree<'a>,
    sort_items: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::OrderBy(OrderBy {
        order: Box::new(order),
        by: Box::new(by),
        sort_items: Box::new(sort_items),
    })
}

#[derive(Clone, Debug)]
pub struct Limit<'a> {
    pub limit: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
}

pub fn limit<'a>(limit: ParseTree<'a>, value: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Limit(Limit {
        limit: Box::new(limit),
        value: Box::new(value),
    })
}

#[derive(Clone, Debug)]
pub struct QuerySetOperation<'a> {
    pub left: Box<ParseTree<'a>>,
    pub operator: Box<ParseTree<'a>>,
    pub set_quantifier_opt: Box<ParseTree<'a>>,
    pub right: Box<ParseTree<'a>>,
}

pub fn query_set_operation<'a>(
    left: ParseTree<'a>,
    operator: ParseTree<'a>,
    set_quantifier_opt: ParseTree<'a>,
    right: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::QuerySetOperation(QuerySetOperation {
        left: Box::new(left),
        operator: Box::new(operator),
        set_quantifier_opt: Box::new(set_quantifier_opt),
        right: Box::new(right),
    })
}
