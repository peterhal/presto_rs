use crate::utils::position;
use std::fmt;

/// A range in a text buffer.
/// End must be at or after start.
/// Start is inclusive, end is exclusive.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TextRange {
    pub start: position::Position,
    pub end: position::Position,
}

impl fmt::Display for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl TextRange {
    pub fn new(start: position::Position, end: position::Position) -> TextRange {
        debug_assert!(start <= end);
        TextRange { start, end }
    }

    /// Constructs an empty TextRange at a position.
    pub fn empty(position: position::Position) -> TextRange {
        TextRange::new(position, position)
    }

    /// Is this the NONE text range?
    pub fn is_none(&self) -> bool {
        self.start == position::START && self.end == position::START
    }

    /// Is position within this range?
    pub fn contains(&self, position: position::Position) -> bool {
        return position >= self.start && position < self.end;
    }

    /// Is range contained within this range?
    pub fn contains_range(&self, range: TextRange) -> bool {
        self.start <= range.start && self.end >= range.end
    }

    // TODO: content_from_lines for error reporting
}

pub const NONE: TextRange = TextRange {
    start: position::START,
    end: position::START,
};
