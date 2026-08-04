use crate::records::ast_expr::AstExpr;

#[repr(C)]
#[derive(Debug)]
pub struct AstExprUnary {
    pub base: AstExpr,
    pub op: AstExprUnaryOp,
    pub expr: *mut AstExpr,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum AstExprUnaryOp {
    Not,
    Minus,
    Len,
}

impl crate::rtti::AstNodeClass for AstExprUnary {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprUnary");
}

#[allow(non_upper_case_globals)]
impl AstExprUnary {
    pub const Not: AstExprUnaryOp = AstExprUnaryOp::Not;
    pub const Minus: AstExprUnaryOp = AstExprUnaryOp::Minus;
    pub const Len: AstExprUnaryOp = AstExprUnaryOp::Len;
}

pub type AstExprUnary_Op = AstExprUnaryOp;
