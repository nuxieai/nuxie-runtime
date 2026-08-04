//! `Lexeme Lexer::lookahead()` — Ast/src/Lexer.cpp:401. Peeks the next token
//! then restores all lexer state (offset/line/lexeme/prevLocation/braceStack).

use crate::enums::brace_type::BraceType;
use crate::records::lexeme::Lexeme;
use crate::records::lexer::Lexer;

impl Lexer {
    pub fn lookahead(&mut self) -> Lexeme {
        let current_offset = self.offset;
        let current_line = self.line;
        let current_line_offset = self.line_offset;
        let current_lexeme = self.lexeme;
        let current_prev_location = self.prev_location;
        let current_brace_stack_size = self.brace_stack.len();
        let current_brace_type = if self.brace_stack.is_empty() {
            BraceType::Normal
        } else {
            *self.brace_stack.last().unwrap()
        };

        let result = *self.next();

        self.offset = current_offset;
        self.line = current_line;
        self.line_offset = current_line_offset;
        self.lexeme = current_lexeme;
        self.prev_location = current_prev_location;

        if self.brace_stack.len() < current_brace_stack_size {
            self.brace_stack.push(current_brace_type);
        } else if self.brace_stack.len() > current_brace_stack_size {
            self.brace_stack.pop();
        }

        result
    }
}
