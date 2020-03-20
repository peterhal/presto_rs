use crate::lexing::{text_range::TextRange, token};

#[derive(Clone, Debug)]
pub enum ParseTree<'a> {
    // The core trees
    Empty(Empty),
    Token(Token<'a>),
    List(List<'a>),
    Error(Error<'a>),

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

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub token: token::Token<'a>,
}

#[derive(Clone, Debug)]
pub struct List<'a> {
    pub start_delimiter: Box<ParseTree<'a>>,
    pub elements: Vec<Box<ParseTree<'a>>>,
    pub separators: Vec<Box<ParseTree<'a>>>,
    pub end_delimiter: Box<ParseTree<'a>>,
}

#[derive(Clone, Debug)]
pub struct Error<'a> {
    pub range: TextRange,
    pub message: &'a str,
}

// The language specific trees

// with_? queryNoWith
#[derive(Clone, Debug)]
pub struct Query<'a> {
    pub with: Box<ParseTree<'a>>,
    pub query_no_with: Box<ParseTree<'a>>,
}

// WITH RECURSIVE? namedQuery (',' namedQuery)*
#[derive(Clone, Debug)]
pub struct With<'a> {
    pub with: Box<ParseTree<'a>>,
    pub recursive: Box<ParseTree<'a>>,
    pub named_queries: Box<ParseTree<'a>>,
}
