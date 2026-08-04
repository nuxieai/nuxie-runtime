//! `Lexeme Lexer::read_comment_body()` — Ast/src/Lexer.cpp:475.

use crate::functions::is_newline::is_newline;
use crate::records::lexeme::{Lexeme, Type};
use crate::records::lexer::Lexer;
use crate::records::location::Location;
use luaur_common::LUAU_ASSERT;

impl Lexer {
    pub(crate) fn read_comment_body(&mut self) -> Lexeme {
        let start = self.position();

        LUAU_ASSERT!(self.peekch_ahead(0) == '-' && self.peekch_ahead(1) == '-');
        self.consume();
        self.consume();

        let start_offset = self.offset;

        if self.peekch() == '[' {
            let sep = self.skip_long_separator();

            if sep >= 0 {
                return self.read_long_string(&start, sep, Type::BlockComment, Type::BrokenComment);
            }
        }

        // fall back to single-line comment
        while self.peekch() != '\0' && self.peekch() != '\r' && !is_newline(self.peekch()) {
            self.consume();
        }

        Lexeme::with_data(
            Location::new(start, self.position()),
            Type::Comment,
            unsafe { self.buffer.add(start_offset as usize) },
            (self.offset - start_offset) as usize,
        )
    }
}
