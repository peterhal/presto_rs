use crate::lexing::text_range;
use std::fmt;

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

#[derive(Copy, Clone, Debug)]
pub struct Comment<'a> {
    pub kind: CommentKind,
    pub range: text_range::TextRange,
    // Includes leading/trailing //, /*, */
    // Includes trailing NewLine for line comments if present
    pub value: &'a str,
}

impl fmt::Display for Comment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "comment({},{},{})", self.kind, self.range, self.value)
    }
}
