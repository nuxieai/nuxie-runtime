use crate::records::ast_array::AstArray;
use crate::records::cst_expr_constant_integer::CstExprConstantInteger;
use crate::records::cst_node::CstNode;
use crate::rtti::CstNodeClass;

impl CstExprConstantInteger {
    pub fn new(value: AstArray<core::ffi::c_char>) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            value,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_expr_constant_integer_cst_expr_constant_integer(
    value: AstArray<core::ffi::c_char>,
) -> CstExprConstantInteger {
    CstExprConstantInteger::new(value)
}
