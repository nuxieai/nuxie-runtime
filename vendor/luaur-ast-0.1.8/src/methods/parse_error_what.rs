//! `ParseError::what` (`Ast/src/Parser.cpp:114`).
//! `const char* what() const noexcept { return message.c_str(); }`.

use crate::records::parse_error::ParseError;

impl ParseError {
    pub fn what(&self) -> &str {
        &self.message
    }
}
