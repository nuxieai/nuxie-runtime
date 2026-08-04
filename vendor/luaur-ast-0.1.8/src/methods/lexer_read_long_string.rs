//! `Lexeme Lexer::read_long_string(const Position& start, int sep, Lexeme::Type ok, Lexeme::Type broken)`
//! — Ast/src/Lexer.cpp:525.

use crate::records::lexeme::{Lexeme, Type};
use crate::records::lexer::Lexer;
use crate::records::location::Location;
use crate::records::position::Position;
use luaur_common::LUAU_ASSERT;

impl Lexer {
    pub(crate) fn read_long_string(
        &mut self,
        start: &Position,
        sep: i32,
        ok: Type,
        broken: Type,
    ) -> Lexeme {
        // skip (second) [
        LUAU_ASSERT!(self.peekch() == '[');
        self.consume();

        let start_offset = self.offset;

        while self.peekch() != '\0' {
            if self.peekch() == ']' {
                if self.skip_long_separator() == sep {
                    LUAU_ASSERT!(self.peekch() == ']');
                    self.consume(); // skip (second) ]

                    let end_offset = self.offset - sep as u32 - 2;
                    LUAU_ASSERT!(end_offset >= start_offset);

                    return Lexeme::with_data(
                        Location::new(*start, self.position()),
                        ok,
                        unsafe { self.buffer.add(start_offset as usize) },
                        (end_offset - start_offset) as usize,
                    );
                }
            } else {
                self.consume_any();
            }
        }

        Lexeme::new(Location::new(*start, self.position()), broken)
    }
}
