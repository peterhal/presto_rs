use crate::lexing::text_range;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Message {
    range: text_range::TextRange,
    message: String,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.range, self.message)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SyntaxError {
    error_code: i32,
    messages: Vec<Message>,
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
