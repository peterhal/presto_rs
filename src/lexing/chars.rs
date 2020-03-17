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
