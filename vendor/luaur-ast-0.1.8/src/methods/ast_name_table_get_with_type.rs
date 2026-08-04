//! `std::pair<AstName, Lexeme::Type> AstNameTable::get_with_type(const char* name, size_t length) const`
//! — Ast/src/Lexer.cpp:245.

use crate::records::ast_name::AstName;
use crate::records::ast_name_table::AstNameTable;
use crate::records::entry::Entry;
use crate::records::lexeme::Type;

impl AstNameTable {
    pub fn get_with_type(&self, name: *const core::ffi::c_char, length: usize) -> (AstName, Type) {
        let key = Entry {
            value: AstName { value: name },
            length: length as u32,
            r#type: Type::Eof,
        };

        if let Some(entry) = self.data.find(&key) {
            (entry.value, entry.r#type)
        } else {
            (AstName::new(), Type::Name)
        }
    }
}
