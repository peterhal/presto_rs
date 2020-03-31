//! Type tests for characters.

pub const NULL: char = '\0';
pub const TAB: char = '\t';
pub const LINE_FEED: char = '\n';
pub const CARRIAGE_RETURN: char = '\r';
pub const SPACE: char = ' ';

pub fn is_whitespace(ch: char) -> bool {
    match ch {
        TAB | LINE_FEED | CARRIAGE_RETURN | SPACE => true,
        _ => false,
    }
}

pub fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

pub fn is_sign(ch: char) -> bool {
    ch == '-' || ch == '+'
}

pub fn is_identifier_start(ch: char) -> bool {
    match ch {
        'a'..='z' | 'A'..='Z' | '_' => true,
        _ => false,
    }
}

pub fn is_identifier_part(ch: char) -> bool {
    match ch {
        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' | '@' | ':' => true,
        _ => false,
    }
}
