//! `Lexeme::Lexeme(const Location&, Type, const char* data, size_t size)` — Ast/src/Lexer.cpp:30.

use crate::records::lexeme::{Lexeme, LexemeData, Type};
use crate::records::location::Location;

impl Lexeme {
    /// A token with a `data`/`length` payload (string, number, comment, ...).
    pub fn with_data(
        location: Location,
        r#type: Type,
        data: *const core::ffi::c_char,
        size: usize,
    ) -> Lexeme {
        luaur_common::LUAU_ASSERT!(
            r#type == Type::RawString
                || r#type == Type::QuotedString
                || r#type == Type::InterpStringBegin
                || r#type == Type::InterpStringMid
                || r#type == Type::InterpStringEnd
                || r#type == Type::InterpStringSimple
                || r#type == Type::BrokenInterpDoubleBrace
                || r#type == Type::Number
                || r#type == Type::Comment
                || r#type == Type::BlockComment
        );

        Lexeme {
            r#type,
            location,
            length: size as u32,
            data: LexemeData { data },
        }
    }
}
