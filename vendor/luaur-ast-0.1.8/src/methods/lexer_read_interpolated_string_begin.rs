//! `Lexeme Lexer::read_interpolated_string_begin()` — Ast/src/Lexer.cpp:616.

use crate::records::lexeme::{Lexeme, Type};
use crate::records::lexer::Lexer;
use luaur_common::LUAU_ASSERT;

impl Lexer {
    pub(crate) fn read_interpolated_string_begin(&mut self) -> Lexeme {
        LUAU_ASSERT!(self.peekch() == '`');

        let start = self.position();
        self.consume();

        self.read_interpolated_string_section(
            start,
            Type::InterpStringBegin,
            Type::InterpStringSimple,
        )
    }
}
