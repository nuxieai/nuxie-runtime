#[repr(C)]
#[derive(Debug)]
pub struct AstStatLocalFunction {
    pub base: crate::records::ast_stat::AstStat,
    pub name: *mut crate::records::ast_local::AstLocal,
    pub func: *mut crate::records::ast_expr_function::AstExprFunction,
    pub is_const: bool,
    /// Position of the `const` keyword; `Position::missing()` when `is_const` is false.
    pub const_keyword_begin: crate::records::position::Position,
}

impl crate::rtti::AstNodeClass for AstStatLocalFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatLocalFunction");
}
