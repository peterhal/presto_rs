use crate::lexing::{chars, position, position::Position, text_range::TextRange};
use std::str::Chars;

/// Represents the current position while lexing some text.
/// Roughly an iterator over a string which tracks the line/column
/// in a Position.
/// Does not include any language specific code.
///
/// peek() operations allow interogating characters at the current location
/// and later. peek()ing beyond the end of the string yields an infinite
/// stream of NULL(0) characters.
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

    pub fn skip_while<P>(&mut self, predicate: P)
    where
        P: Fn(char) -> bool,
    {
        while predicate(self.peek()) {
            self.next();
        }
    }

    fn chars(&self) -> Chars<'a> {
        // TODO: from_utf8_unchecked
        if let Ok(s) = std::str::from_utf8(self.source.as_bytes()) {
            s[self.index..].chars()
        } else {
            panic!("bad str")
        }
    }

    pub fn peek(&self) -> char {
        self.chars().next().unwrap_or(chars::NULL)
    }

    pub fn peek_offset(&self, mut offset: i32) -> char {
        assert!(offset >= 0);

        let mut clone = self.clone();
        while offset > 0 {
            clone.next();
            offset -= 1;
        }
        clone.peek()
    }

    pub fn peek_char(&self, ch: char) -> bool {
        self.peek() == ch
    }

    pub fn peek_char_offset(&self, ch: char, offset: i32) -> bool {
        self.peek_offset(offset) == ch
    }

    pub fn at_end(&self) -> bool {
        self.peek_char(chars::NULL)
    }

    fn advance_index_of_char(&mut self, ch: Option<char>) {
        if let Some(ch) = ch {
            self.index += ch.len_utf8();
        }
    }

    pub fn next(&mut self) -> char {
        let ch = self.chars().next();
        self.advance_index_of_char(ch);
        self.position = match ch {
            Some(chars::LINE_FEED) => self.position.next_line(),
            Some(chars::CARRIAGE_RETURN) => {
                // handle windows line endings
                if self.peek_char(chars::LINE_FEED) {
                    self.advance_index_of_char(self.chars().next());
                }
                self.position.next_line()
            }
            Some(_) => self.position.next_column(),
            None => self.position,
        };
        ch.unwrap_or(chars::NULL)
    }

    pub fn get_range(&self, end: &LexerPosition<'a>) -> TextRange {
        assert!(end.index >= self.index);
        TextRange {
            start: self.position,
            end: end.position,
        }
    }

    pub fn get_text(&self, end: &LexerPosition<'a>) -> &'a str {
        assert!(end.index >= self.index);
        &self.source[self.index..end.index]
    }
}
