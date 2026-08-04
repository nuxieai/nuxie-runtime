//! `const Lexeme& Lexer::next()` — Ast/src/Lexer.cpp:369.

use crate::records::lexeme::Lexeme;
use crate::records::lexer::Lexer;

impl Lexer {
    pub fn next(&mut self) -> &Lexeme {
        let skip_comments = self.skip_comments;
        self.next_with(skip_comments, true)
    }
}
