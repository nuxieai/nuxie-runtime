//! `void Lexer::nextline()` — Ast/src/Lexer.cpp:393.

use crate::functions::is_newline::is_newline;
use crate::records::lexer::Lexer;

impl Lexer {
    pub fn nextline(&mut self) {
        while self.peekch() != '\0' && self.peekch() != '\r' && !is_newline(self.peekch()) {
            self.consume();
        }

        self.next();
    }
}
