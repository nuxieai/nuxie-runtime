//! `char Lexer::peekch(unsigned int lookahead) const` — Ast/src/Lexer.cpp:442.

use crate::records::lexer::Lexer;

impl Lexer {
    #[inline(always)]
    pub(crate) fn peekch_ahead(&self, lookahead: u32) -> char {
        if ((self.offset + lookahead) as usize) < self.buffer_size {
            unsafe { *self.buffer.add((self.offset + lookahead) as usize) as u8 as char }
        } else {
            '\0'
        }
    }
}
