//! Utilities shared by other modules.

pub use self::position::Position;
pub use self::syntax_error::Message;
pub use self::syntax_error::SyntaxError;
pub use self::text_range::TextRange;

pub mod position;
pub mod syntax_error;
pub mod text_range;
