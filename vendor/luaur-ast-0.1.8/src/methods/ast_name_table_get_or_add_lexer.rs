//! `AstName AstNameTable::get_or_add(const char* name, size_t len)` — Ast/src/Lexer.cpp:254.

use crate::records::ast_name::AstName;
use crate::records::ast_name_table::AstNameTable;

impl AstNameTable {
    pub fn get_or_add(&mut self, name: *const core::ffi::c_char, len: usize) -> AstName {
        self.get_or_add_with_type(name, len).0
    }
}
