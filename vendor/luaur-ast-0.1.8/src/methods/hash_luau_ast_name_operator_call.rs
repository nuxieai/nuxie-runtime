use crate::records::ast_name::AstName;
use crate::records::hash_luau_ast_name::hash_AstName;

impl hash_AstName {
    pub fn operator_call(&self, value: &AstName) -> usize {
        let ptr = value.value as usize;
        (ptr >> 4) ^ (ptr >> 9)
    }
}
