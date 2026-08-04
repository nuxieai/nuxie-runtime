//! `char Lexer::peekch() const` — Ast/src/Lexer.cpp:436.

use crate::records::lexer::Lexer;

impl Lexer {
    #[inline(always)]
    pub(crate) fn peekch(&self) -> char {
        if (self.offset as usize) < self.buffer_size {
            unsafe { *self.buffer.add(self.offset as usize) as u8 as char }
        } else {
            '\0'
        }
    }
}
