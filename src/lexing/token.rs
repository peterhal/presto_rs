use crate::lexing::{
    comment::Comment, syntax_error::SyntaxError, text_range::TextRange, token_kind::TokenKind,
};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub range: TextRange,
    pub value: &'a str,
    pub leading_comments: Vec<Comment<'a>>,
    pub trailing_comments: Vec<Comment<'a>>,
    pub errors: Vec<SyntaxError>,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> Token<'a> {
    pub fn full_range(&self) -> TextRange {
        let start = if let Some(comment) = self.leading_comments.last() {
            comment.range.start
        } else {
            self.range.start
        };
        let end = if let Some(comment) = self.trailing_comments.last() {
            comment.range.end
        } else {
            self.range.end
        };
        TextRange::new(start, end)
    }
}
