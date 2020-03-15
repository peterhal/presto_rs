use crate::lexing::text_range;

enum CommentKind {
    LineComment,
    DelimitedComment,
}

struct Comment {
    kind: CommentKind,
    range: text_range::TextRange,
    // Includes leading/trailing //, /*, */
    // Includes trailing NewLine for line comments if present
    value: String,
}
