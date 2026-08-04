#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct AstExprInterpString {
    pub base: crate::records::ast_expr::AstExpr,
    pub strings:
        crate::records::ast_array::AstArray<crate::records::ast_array::AstArray<core::ffi::c_char>>,
    pub expressions: crate::records::ast_array::AstArray<*mut crate::records::ast_expr::AstExpr>,
}

impl crate::rtti::AstNodeClass for AstExprInterpString {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprInterpString");
}
