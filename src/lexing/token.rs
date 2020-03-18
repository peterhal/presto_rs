use crate::lexing::{
    comment::Comment, syntax_error::SyntaxError, text_range::TextRange, token_kind::TokenKind,
};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub range: TextRange,
    pub value: &'a str,
    pub leading_comments: Vec<Comment>,
    pub trailing_comments: Vec<Comment>,
    pub errors: Vec<SyntaxError>,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token({},{},{})", self.kind, self.range, self.value)
    }
}
