use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstExprConstantBool {
    pub base: AstExpr,
    pub value: bool,
}

impl crate::rtti::AstNodeClass for AstExprConstantBool {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprConstantBool");
}

#[allow(non_snake_case)]
impl AstExprConstantBool {
    pub const fn LUAU_RTTI() {
        crate::macros::luau_rtti::LUAU_RTTI::<Self>()
    }
}
