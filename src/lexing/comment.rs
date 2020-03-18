use crate::lexing::text_range;

#[derive(Copy, Clone, Debug)]
pub enum CommentKind {
    LineComment,
    DelimitedComment,
}

#[derive(Clone, Debug)]
pub struct Comment {
    kind: CommentKind,
    range: text_range::TextRange,
    // Includes leading/trailing //, /*, */
    // Includes trailing NewLine for line comments if present
    value: String,
}
