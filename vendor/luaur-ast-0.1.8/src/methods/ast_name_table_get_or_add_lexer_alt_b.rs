//! `AstName AstNameTable::get_or_add(const char* name)` — Ast/src/Lexer.cpp:259.

use crate::records::ast_name::AstName;
use crate::records::ast_name_table::AstNameTable;

impl AstNameTable {
    pub fn get_or_add_c_str(&mut self, name: *const core::ffi::c_char) -> AstName {
        let len = unsafe { core::ffi::CStr::from_ptr(name).to_bytes().len() };
        self.get_or_add_with_type(name, len).0
    }
}
