//! `Lexeme Lexer::read_number(const Position& start, unsigned int startOffset)`
//! — Ast/src/Lexer.cpp:679.

use crate::functions::is_alpha::is_alpha;
use crate::functions::is_digit_lexer::is_digit;
use crate::records::lexeme::{Lexeme, Type};
use crate::records::lexer::Lexer;
use crate::records::location::Location;
use crate::records::position::Position;
use luaur_common::LUAU_ASSERT;

impl Lexer {
    pub(crate) fn read_number(&mut self, start: &Position, start_offset: u32) -> Lexeme {
        LUAU_ASSERT!(is_digit(self.peekch()));

        // This function does not do the number parsing - it only skips a
        // number-like pattern. The resulting string is later converted to a
        // number with proper verification.
        loop {
            self.consume();
            if !(is_digit(self.peekch()) || self.peekch() == '.' || self.peekch() == '_') {
                break;
            }
        }

        if self.peekch() == 'e' || self.peekch() == 'E' {
            self.consume();

            if self.peekch() == '+' || self.peekch() == '-' {
                self.consume();
            }
        }

        while is_alpha(self.peekch()) || is_digit(self.peekch()) || self.peekch() == '_' {
            self.consume();
        }

        Lexeme::with_data(
            Location::new(*start, self.position()),
            Type::Number,
            unsafe { self.buffer.add(start_offset as usize) },
            (self.offset - start_offset) as usize,
        )
    }
}
