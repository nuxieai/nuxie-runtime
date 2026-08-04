use crate::records::ast_expr::AstExpr;
use crate::records::ast_name::AstName;
use crate::records::location::Location;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstExprIndexName {
    pub base: AstExpr,
    pub expr: *mut AstExpr,
    pub index: AstName,
    pub index_location: Location,
    pub op_position: Position,
    pub op: core::ffi::c_char,
}

impl crate::rtti::AstNodeClass for AstExprIndexName {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprIndexName");
}

#[allow(non_upper_case_globals)]
impl AstExprIndexName {
    pub const ClassIndex: i32 = <Self as crate::rtti::AstNodeClass>::CLASS_INDEX;
}

#[allow(non_snake_case)]
pub const fn LUAU_RTTI_AstExprIndexName() {
    crate::macros::luau_rtti::LUAU_RTTI::<AstExprIndexName>();
}
