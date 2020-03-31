use crate::lexing::{comment::Comment, token_kind::TokenKind};
use crate::utils::{position::Position, syntax_error::SyntaxError, text_range::TextRange};
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
    pub fn full_start(&self) -> Position {
        if let Some(comment) = self.leading_comments.last() {
            comment.range.start
        } else {
            self.range.start
        }
    }

    pub fn full_end(&self) -> Position {
        if let Some(comment) = self.trailing_comments.last() {
            comment.range.end
        } else {
            self.range.end
        }
    }

    pub fn full_range(&self) -> TextRange {
        TextRange::new(self.full_start(), self.full_end())
    }
}
