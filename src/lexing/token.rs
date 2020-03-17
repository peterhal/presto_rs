use crate::lexing::{
    comment::Comment, syntax_error::SyntaxError, text_range::TextRange, token_kind::TokenKind,
};

pub struct Token {
    kind: TokenKind,
    range: TextRange,
    value: String,
    leading_comments: Vec<Comment>,
    trailing_comments: Vec<Comment>,
    errors: Vec<SyntaxError>,
}
