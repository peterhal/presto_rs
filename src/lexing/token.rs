use super::{Comment, TokenKind};
use crate::utils::{Position, SyntaxError, TextRange};
use std::fmt;

/// A lexical token (aka lexeme) in a query.
///
/// A token's lifetime is typically scoped
/// to the lifetime of the text being parsed.
///
/// A token includes leading and trailing comments.
/// Trailing comments are any comments that start
/// on the same line as the end of the token, when the token
/// is the last token on a line. Otherwise the comment
/// will be a leading comment of the next token.
///
/// Tokens also contain syntax errors. Tokens whose kind is not
/// Error may still contain errors. To accumulate all errors
/// from a set of tokens every token's errors list must be unioned.
///
/// A token's value includes the part of the text significant in the
/// language. It does not include the comment, whitespace or error text.
///
/// A token's range is the range of the token's value; it also
/// does not include the range of comments, whitespace or error text.
/// Use full_start/full_end/ful_range to get the range of the token
/// which includes trivia (leading/training comments).
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
    /// The starting position including trivia.
    pub fn full_start(&self) -> Position {
        if let Some(comment) = self.leading_comments.last() {
            comment.range.start
        } else {
            self.range.start
        }
    }

    /// The ending position including trivia.
    pub fn full_end(&self) -> Position {
        if let Some(comment) = self.trailing_comments.last() {
            comment.range.end
        } else {
            self.range.end
        }
    }

    /// Token's range including trivia.
    pub fn full_range(&self) -> TextRange {
        TextRange::new(self.full_start(), self.full_end())
    }
}
