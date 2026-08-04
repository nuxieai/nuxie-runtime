//! `Lexer::Lexer(const char* buffer, size_t buffer_size, AstNameTable& names, Position startPosition)`
//! — Ast/src/Lexer.cpp:346.

use crate::records::ast_name_table::AstNameTable;
use crate::records::lexeme::{Lexeme, Type};
use crate::records::lexer::Lexer;
use crate::records::location::Location;
use crate::records::position::Position;

impl Lexer {
    pub fn new(
        buffer: *const core::ffi::c_char,
        buffer_size: usize,
        names: &mut AstNameTable,
        start_position: Position,
    ) -> Lexer {
        Lexer {
            buffer,
            buffer_size,
            offset: 0,
            line: start_position.line,
            // `lineOffset(0u - startPosition.column)` — wrapping so that
            // `position()` reports `offset - lineOffset` == the start column.
            line_offset: 0u32.wrapping_sub(start_position.column),
            lexeme: Lexeme::new(
                Location::with_length(
                    Position {
                        line: start_position.line,
                        column: start_position.column,
                    },
                    0,
                ),
                Type::Eof,
            ),
            prev_location: Location::default(),
            names: names as *mut AstNameTable,
            skip_comments: false,
            read_names: true,
            brace_stack: alloc::vec::Vec::new(),
        }
    }
}
