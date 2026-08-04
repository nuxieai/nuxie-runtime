//! `std::pair<AstName, Lexeme::Type> Lexer::read_name()` — Ast/src/Lexer.cpp:705.

use crate::functions::is_alpha::is_alpha;
use crate::functions::is_digit_lexer::is_digit;
use crate::records::ast_name::AstName;
use crate::records::ast_name_table::AstNameTable;
use crate::records::lexeme::Type;
use crate::records::lexer::Lexer;
use luaur_common::LUAU_ASSERT;

impl Lexer {
    pub(crate) fn read_name(&mut self) -> (AstName, Type) {
        LUAU_ASSERT!(is_alpha(self.peekch()) || self.peekch() == '_' || self.peekch() == '@');

        let start_offset = self.offset;

        loop {
            self.consume();
            if !(is_alpha(self.peekch()) || is_digit(self.peekch()) || self.peekch() == '_') {
                break;
            }
        }

        let read_names = self.read_names;
        let data = unsafe { self.buffer.add(start_offset as usize) };
        let length = (self.offset - start_offset) as usize;
        let names: &mut AstNameTable = unsafe { &mut *self.names };

        if read_names {
            names.get_or_add_with_type(data, length)
        } else {
            names.get_with_type(data, length)
        }
    }
}
