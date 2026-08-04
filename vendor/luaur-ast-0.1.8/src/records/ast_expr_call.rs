use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprCall {
    pub base: AstExpr,
    pub func: *mut AstExpr,
    pub type_arguments: AstArray<AstTypeOrPack>,
    pub args: AstArray<*mut AstExpr>,
    pub self_: bool,
    pub arg_location: Location,
}

impl crate::rtti::AstNodeClass for AstExprCall {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprCall");
}

#[allow(non_snake_case)]
impl AstExprCall {
    pub const fn LUAU_RTTI(&self) {
        crate::macros::luau_rtti::LUAU_RTTI::<Self>()
    }
}
