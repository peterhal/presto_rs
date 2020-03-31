use super::chars;
use crate::utils::{position, Position, TextRange};
use std::str::Chars;

/// Represents the current position while lexing some text.
/// Roughly an iterator over a string which tracks the line/column
/// in a Position.
///
/// Does not include any language specific code.
///
/// peek() operations allow interogating characters at or after
/// the current location. peek()ing beyond the end of the string yields
/// an infinite stream of NULL(0) characters.
///
/// index indexes into the source string in bytes *not* chars.
///
/// The lifetime of LexerPosition is scoped to the lifetime of the
/// source string.
#[derive(Clone, Copy, Debug)]
pub struct LexerPosition<'a> {
    pub source: &'a str,
    pub position: Position,
    // index into source in bytes
    pub index: usize,
}

impl<'a> LexerPosition<'a> {
    pub fn new(source: &'a str) -> LexerPosition<'a> {
        LexerPosition {
            source,
            position: position::START,
            index: 0,
        }
    }

    pub fn line(&self) -> i32 {
        self.position.line
    }

    pub fn skip_while<P>(&mut self, predicate: P) -> bool
    where
        P: Fn(char) -> bool,
    {
        let mut result = false;
        while predicate(self.peek()) {
            self.next();
            result = true
        }
        result
    }

    /// An iterator over the remaining chars in source.
    /// This is a constant time operation.
    fn chars(&self) -> Chars<'a> {
        // This is safe as the source was checked for utf8 correctness
        // before self was created; and self.index is always advanced on char
        // boundaries.
        unsafe { std::str::from_utf8_unchecked(self.source.as_bytes())[self.index..].chars() }
    }

    /// Returns the next char in the source or None if at end.
    /// Does not consume the char.
    /// This is a constant time operation.
    fn peek_char_opt(&self) -> Option<char> {
        self.chars().next()
    }

    /// Returns the next char in the source.
    /// Does not consume the char.
    /// This is a constant time operation.
    pub fn peek(&self) -> char {
        self.peek_char_opt().unwrap_or(chars::NULL)
    }

    /// Returns the char at offset from the current location in the source.
    /// This is an O(offset) operation; fortunately offset is rarely > 2.
    pub fn peek_offset(&self, mut offset: i32) -> char {
        debug_assert!(offset >= 0);

        let mut clone = self.clone();
        while offset > 0 {
            clone.next();
            offset -= 1;
        }
        clone.peek()
    }

    /// Returns true if the next char matches ch.
    /// Does not consume input.
    pub fn peek_char(&self, ch: char) -> bool {
        self.peek() == ch
    }

    /// Returns true if the char at offset matches ch.
    /// Does not consume input.
    pub fn peek_char_offset(&self, ch: char, offset: i32) -> bool {
        self.peek_offset(offset) == ch
    }

    /// Are we at the end of the source.
    pub fn at_end(&self) -> bool {
        self.peek_char(chars::NULL)
    }

    fn advance_index_of_char(&mut self, ch: Option<char>) {
        if let Some(ch) = ch {
            self.index += ch.len_utf8();
        }
    }

    fn advance_position_of_char(&mut self, ch: Option<char>) {
        self.position = match ch {
            Some(chars::LINE_FEED) => self.position.next_line(),
            Some(chars::CARRIAGE_RETURN) => {
                // handle windows line endings
                if self.peek_char(chars::LINE_FEED) {
                    self.advance_index_of_char(self.peek_char_opt());
                }
                self.position.next_line()
            }
            Some(_) => self.position.next_column(),
            None => self.position,
        };
    }

    /// Returns the next char in the input.
    /// Advances past the consumed char.
    pub fn next(&mut self) -> char {
        let ch = self.peek_char_opt();
        self.advance_index_of_char(ch);
        self.advance_position_of_char(ch);
        ch.unwrap_or(chars::NULL)
    }
}

pub fn get_range<'a>(&start: &LexerPosition<'a>, end: &LexerPosition<'a>) -> TextRange {
    debug_assert!(end.index >= start.index);
    TextRange::new(start.position, end.position)
}

pub fn get_text<'a>(&start: &LexerPosition<'a>, end: &LexerPosition<'a>) -> &'a str {
    debug_assert!(end.index >= start.index);
    &start.source[start.index..end.index]
}
