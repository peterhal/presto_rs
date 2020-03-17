use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Position {
    line: i32,
    column: i32,
}

impl Position {
    pub fn at_beginning_of_line(&self) -> bool {
        self.column == 0
    }

    pub fn next_line(&self) -> Position {
        Position {
            line: self.line + 1,
            column: 0,
        }
    }

    pub fn next_column(&self) -> Position {
        Position {
            line: self.line,
            column: self.column + 1,
        }
    }

    pub fn beginning_of_line(&self) -> Position {
        Position {
            line: self.line,
            column: 0,
        }
    }

    // TODO: create_end_of_lines
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.line, self.column)
    }
}

pub const START: Position = Position { line: 0, column: 0 };
