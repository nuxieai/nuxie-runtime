use crate::records::ast_name::AstName;

impl AstName {
    pub fn operator_eq_ast_name(&self, rhs: &AstName) -> bool {
        self.value == rhs.value
    }
}
