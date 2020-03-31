use crate::utils::text_range;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Message {
    pub range: text_range::TextRange,
    pub message: String,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.range, self.message)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SyntaxError {
    pub error_code: i32,
    pub messages: Vec<Message>,
    // TODO: fix
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error {} @{}",
            self.error_code,
            self.messages
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

// lex errors 100-199
pub const ERROR_EXPECTED_CHAR: i32 = 101;
pub const ERROR_UNTERMINATED_DELIMITED_COMMENT: i32 = 102;
pub const ERROR_INVALID_TOKEN_START: i32 = 103;
pub const ERROR_UNTERMINATED_STRING_LITERAL: i32 = 104;
pub const ERROR_UNTERMINATED_QUOTED_IDENTIFIER: i32 = 105;
pub const ERROR_UNTERMINATED_BACK_QUOTED_IDENTIFIER: i32 = 106;

// parse errors 200-299
pub const ERROR_SYNTAX_ERROR: i32 = 201;
