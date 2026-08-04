use crate::records::ast_name::AstName;

impl AstName {
    pub fn operator_ne_ast_name(&self, rhs: &AstName) -> bool {
        self.value != rhs.value
    }
}
