//! Faithful port of Luau `Lexeme` (`Ast/include/Luau/Lexer.h`).
//!
//! The token payload is a C++ union (`const char* data`/`name`, `unsigned
//! codepoint`); a Rust `union` reproduces it. Unions can derive `Clone`/`Copy`
//! but not `Debug`, so `LexemeData` gets a hand-written `Debug` (it can't know
//! which arm is active) and `Lexeme` then derives `Debug` normally.

pub use crate::enums::type_lexer::Type;
use crate::records::location::Location;

/// `Lexeme::QuoteStyle` (`Ast/include/Luau/Lexer.h`) — the delimiter of a quoted
/// string token, returned by `get_quote_style`.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuoteStyle {
    Single,
    Double,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy)]
pub struct Lexeme {
    pub r#type: Type,
    pub location: Location,
    pub(crate) length: u32,
    pub data: LexemeData,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union LexemeData {
    pub data: *const core::ffi::c_char,
    pub name: *const core::ffi::c_char,
    pub codepoint: u32,
}

impl core::fmt::Debug for LexemeData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // The active arm is determined by `Lexeme::type`; print opaquely.
        f.write_str("LexemeData(..)")
    }
}

impl Default for LexemeData {
    fn default() -> Self {
        Self {
            data: core::ptr::null(),
        }
    }
}
