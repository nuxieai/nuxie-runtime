//! `void Lexer::read_backslash_in_string()` — Ast/src/Lexer.cpp:557.

use crate::functions::is_space::is_space;
use crate::records::lexer::Lexer;
use luaur_common::LUAU_ASSERT;

impl Lexer {
    pub(crate) fn read_backslash_in_string(&mut self) {
        LUAU_ASSERT!(self.peekch() == '\\');
        self.consume();
        match self.peekch() {
            '\r' => {
                self.consume();
                if self.peekch() == '\n' {
                    self.consume_any();
                }
            }
            '\0' => {}
            'z' => {
                self.consume();
                while is_space(self.peekch()) {
                    self.consume_any();
                }
            }
            _ => self.consume_any(),
        }
    }
}
