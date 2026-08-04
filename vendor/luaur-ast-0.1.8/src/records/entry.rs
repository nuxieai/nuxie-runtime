use crate::enums::type_lexer::Type;
use crate::records::ast_name::AstName;

#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub value: AstName,
    pub length: u32,
    pub r#type: Type,
}

impl Entry {
    pub const fn new(value: AstName, length: u32, r#type: Type) -> Self {
        Self {
            value,
            length,
            r#type,
        }
    }
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            value: AstName {
                value: core::ptr::null(),
            },
            length: 0,
            r#type: Type::Eof,
        }
    }
}
