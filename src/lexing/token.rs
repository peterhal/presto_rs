use crate::lexing::{
    comment::Comment, syntax_error::SyntaxError, text_range::TextRange, token_kind::TokenKind,
};

pub struct Token<'a> {
    pub kind: TokenKind,
    pub range: TextRange,
    pub value: &'a str,
    pub leading_comments: Vec<Comment>,
    pub trailing_comments: Vec<Comment>,
    pub errors: Vec<SyntaxError>,
}
