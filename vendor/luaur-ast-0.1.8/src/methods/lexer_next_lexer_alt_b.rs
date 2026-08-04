//! `const Lexeme& Lexer::next(bool skipComments, bool updatePrevLocation)`
//! — Ast/src/Lexer.cpp:374.

use crate::functions::is_space::is_space;
use crate::records::lexeme::{Lexeme, Type};
use crate::records::lexer::Lexer;

impl Lexer {
    pub fn next_with(&mut self, skip_comments: bool, mut update_prev_location: bool) -> &Lexeme {
        // in skipComments mode we reject valid comments
        loop {
            // consume whitespace before the token
            while is_space(self.peekch()) {
                self.consume_any();
            }

            if update_prev_location {
                self.prev_location = self.lexeme.location;
            }

            self.lexeme = self.read_next();
            update_prev_location = false;

            if !(skip_comments
                && (self.lexeme.r#type == Type::Comment
                    || self.lexeme.r#type == Type::BlockComment))
            {
                break;
            }
        }

        &self.lexeme
    }
}
