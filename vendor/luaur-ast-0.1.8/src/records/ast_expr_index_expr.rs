use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::rtti::AstNodeClass;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprIndexExpr {
    pub base: AstExpr,
    pub expr: *mut AstExpr,
    pub index: *mut AstExpr,
}

impl AstNodeClass for AstExprIndexExpr {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprIndexExpr");
}
