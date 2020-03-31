use crate::utils::TextRange;
use std::fmt;

/// The kinds of comment trivia.
#[derive(Copy, Clone, Debug)]
pub enum CommentKind {
    LineComment,
    DelimitedComment,
}

impl fmt::Display for CommentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A comment from source text.
///
/// The lifetime of a Comment is typically scoped to the
/// lifetime of the source text being lexed.
///
/// For range and value, includes leading/trailing //, /*, */.
/// Includes trailing NewLine for line comments if present.
#[derive(Copy, Clone, Debug)]
pub struct Comment<'a> {
    pub kind: CommentKind,
    pub range: TextRange,
    pub value: &'a str,
}

impl fmt::Display for Comment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "comment({},{},{})", self.kind, self.range, self.value)
    }
}
