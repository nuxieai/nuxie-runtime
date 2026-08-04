use crate::records::ast_type::AstType;

impl AstType {
    pub fn as_type(&mut self) -> *mut AstType {
        self as *mut Self
    }
}
