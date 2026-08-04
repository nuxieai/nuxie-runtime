//! `void Lexer::set_skip_comments(bool skip)` — Ast/src/Lexer.cpp:359.

use crate::records::lexer::Lexer;

impl Lexer {
    pub fn set_skip_comments(&mut self, skip: bool) {
        self.skip_comments = skip;
    }
}
