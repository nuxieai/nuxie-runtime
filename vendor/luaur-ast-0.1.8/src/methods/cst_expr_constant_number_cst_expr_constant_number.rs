use crate::records::ast_array::AstArray;
use crate::records::cst_expr_constant_number::CstExprConstantNumber;
use crate::records::cst_node::CstNode;
use crate::rtti::CstNodeClass;

impl CstExprConstantNumber {
    #[allow(non_snake_case)]
    pub fn new(value: AstArray<core::ffi::c_char>) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            value,
        }
    }
}

pub fn cst_expr_constant_number_cst_expr_constant_number(
    value: &AstArray<core::ffi::c_char>,
) -> CstExprConstantNumber {
    CstExprConstantNumber::new(*value)
}
