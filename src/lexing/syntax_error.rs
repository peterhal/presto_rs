use crate::lexing::text_range;
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

pub const ERROR_EXPECTED_CHAR: i32 = 101;
pub const ERROR_UNTERMINATED_DELIMITED_COMMENT: i32 = 102;
