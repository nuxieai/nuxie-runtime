//! `Lexeme Lexer::read_utf_8_error()` — Ast/src/Lexer.cpp:1048.

use crate::records::lexeme::{Lexeme, LexemeData, Type};
use crate::records::lexer::Lexer;
use crate::records::location::Location;

impl Lexer {
    pub(crate) fn read_utf_8_error(&mut self) -> Lexeme {
        let start = self.position();
        let mut codepoint: u32 = 0;
        let mut size: i32 = 0;

        if ((self.peekch() as u8) & 0b1000_0000) == 0b0000_0000 {
            size = 1;
            codepoint = ((self.peekch() as u8) & 0x7F) as u32;
        } else if ((self.peekch() as u8) & 0b1110_0000) == 0b1100_0000 {
            size = 2;
            codepoint = ((self.peekch() as u8) & 0b1_1111) as u32;
        } else if ((self.peekch() as u8) & 0b1111_0000) == 0b1110_0000 {
            size = 3;
            codepoint = ((self.peekch() as u8) & 0b1111) as u32;
        } else if ((self.peekch() as u8) & 0b1111_1000) == 0b1111_0000 {
            size = 4;
            codepoint = ((self.peekch() as u8) & 0b111) as u32;
        } else {
            self.consume();
            return Lexeme::new(Location::new(start, self.position()), Type::BrokenUnicode);
        }

        self.consume();

        let mut i = 1;
        while i < size {
            if ((self.peekch() as u8) & 0b1100_0000) != 0b1000_0000 {
                return Lexeme::new(Location::new(start, self.position()), Type::BrokenUnicode);
            }

            codepoint <<= 6;
            codepoint |= ((self.peekch() as u8) & 0b0011_1111) as u32;
            self.consume();
            i += 1;
        }

        let mut result = Lexeme::new(Location::new(start, self.position()), Type::BrokenUnicode);
        result.data = LexemeData { codepoint };
        result
    }
}
