//! `AstName AstNameTable::get(const char* name) const` — Ast/src/Lexer.cpp:264.

use crate::records::ast_name::AstName;
use crate::records::ast_name_table::AstNameTable;

impl AstNameTable {
    pub fn get(&self, name: *const core::ffi::c_char) -> AstName {
        let len = unsafe { core::ffi::CStr::from_ptr(name).to_bytes().len() };
        self.get_with_type(name, len).0
    }
}
