use crate::lexing::{text_range::TextRange, token};

#[derive(Clone, Debug)]
pub enum ParseTree<'a> {
    // The core trees
    Empty(Empty),
    Token(Token<'a>),
    List(List<'a>),
    Error(Error),

    // The language specific trees

    // with_? queryNoWith
    Query(Query<'a>),
    // WITH RECURSIVE? namedQuery (',' namedQuery)*
    With(With<'a>),
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

pub fn error(range: TextRange, message: &str) -> ParseTree {
    ParseTree::Error(Error {
        range,
        message: message.to_string(),
    })
}

// The language specific trees

// with_? queryNoWith
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

// WITH RECURSIVE? namedQuery (',' namedQuery)*
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
