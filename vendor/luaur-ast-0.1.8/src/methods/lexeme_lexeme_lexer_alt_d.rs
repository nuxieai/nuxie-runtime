//! `Lexeme::Lexeme(const Location&, Type, const char* name)` — Ast/src/Lexer.cpp:42.

use crate::records::lexeme::{Lexeme, LexemeData, Type};
use crate::records::location::Location;

impl Lexeme {
    /// A name/attribute/reserved-word token: the `name` union arm, `length = 0`.
    pub fn with_name(location: Location, r#type: Type, name: *const core::ffi::c_char) -> Lexeme {
        luaur_common::LUAU_ASSERT!(
            r#type == Type::Name
                || r#type == Type::Attribute
                || (r#type >= Type::Reserved_BEGIN && r#type < Type::Reserved_END)
        );

        Lexeme {
            r#type,
            location,
            length: 0,
            data: LexemeData { name },
        }
    }
}
