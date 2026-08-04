//! `int Lexer::skip_long_separator()` — Ast/src/Lexer.cpp:507.
//!
//! Given a sequence `[===[` or `]===]`, returns the number of `=` signs (or 0),
//! `-1` if not a long separator, or `-N` if malformed. Does not consume the
//! closing brace.

use crate::records::lexer::Lexer;
use luaur_common::LUAU_ASSERT;

impl Lexer {
    pub(crate) fn skip_long_separator(&mut self) -> i32 {
        let start = self.peekch();

        LUAU_ASSERT!(start == '[' || start == ']');
        self.consume();

        let mut count = 0i32;

        while self.peekch() == '=' {
            self.consume();
            count += 1;
        }

        if start == self.peekch() {
            count
        } else {
            -count - 1
        }
    }
}
