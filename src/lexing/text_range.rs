use std::fmt;
use crate::lexing::position;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TextRange {
    start: position::Position,
    end: position::Position,
}

impl fmt::Display for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_none() {
            write!(f, "")
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

impl TextRange {
    fn is_none(&self) -> bool {
        self.start == position::START && self.end == position::START
    }

    fn contains(&self, position: position::Position) -> bool {
        return position >= self.start && position <= self.end
    }

    fn contains_range(&self, range: TextRange) -> bool {
        self.start <= range.start && self.end >= range.end
    }

    // TODO: content_from_lines
}

pub const NONE: TextRange = TextRange{start: position::START, end: position::START};
