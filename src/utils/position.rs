use std::fmt;

/// A position within a text buffer.
/// Both line and column use 0 based indexes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Position {
    pub line: i32,
    pub column: i32,
}

impl Position {
    pub fn new(line: i32, column: i32) -> Position {
        debug_assert!(line >= 0 && column >= 0);
        Position { line, column }
    }

    pub fn at_beginning_of_line(&self) -> bool {
        self.column == 0
    }

    /// Returns a new Position at the start of the next line.
    pub fn next_line(&self) -> Position {
        Position::new(self.line + 1, 0)
    }

    /// Returns a new Position at the start of the next column.
    pub fn next_column(&self) -> Position {
        Position::new(self.line, self.column + 1)
    }

    /// Returns a new Position at the start of the current line.
    pub fn beginning_of_line(&self) -> Position {
        Position::new(self.line, 0)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.line, self.column)
    }
}

/// The Position indicating the start of a text buffer.
pub const START: Position = Position { line: 0, column: 0 };
