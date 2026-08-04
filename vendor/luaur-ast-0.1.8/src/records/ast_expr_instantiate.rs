use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_type_or_pack::AstTypeOrPack;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprInstantiate {
    pub base: AstExpr,
    pub expr: *mut AstExpr,
    pub type_arguments: AstArray<AstTypeOrPack>,
}

impl crate::rtti::AstNodeClass for AstExprInstantiate {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprInstantiate");
}
