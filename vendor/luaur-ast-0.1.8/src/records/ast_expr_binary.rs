use crate::records::ast_expr::AstExpr;

#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct AstExprBinary {
    pub base: AstExpr,
    pub op: AstExprBinary_Op,
    pub left: *mut AstExpr,
    pub right: *mut AstExpr,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum AstExprBinary_Op {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    Concat,
    CompareNe,
    CompareEq,
    CompareLt,
    CompareLe,
    CompareGt,
    CompareGe,
    And,
    Or,
    Op__Count,
}

impl crate::rtti::AstNodeClass for AstExprBinary {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprBinary");
}

impl AstExprBinary {
    pub const Add: AstExprBinary_Op = AstExprBinary_Op::Add;
    pub const Sub: AstExprBinary_Op = AstExprBinary_Op::Sub;
    pub const Mul: AstExprBinary_Op = AstExprBinary_Op::Mul;
    pub const Div: AstExprBinary_Op = AstExprBinary_Op::Div;
    pub const FloorDiv: AstExprBinary_Op = AstExprBinary_Op::FloorDiv;
    pub const Mod: AstExprBinary_Op = AstExprBinary_Op::Mod;
    pub const Pow: AstExprBinary_Op = AstExprBinary_Op::Pow;
    pub const Concat: AstExprBinary_Op = AstExprBinary_Op::Concat;
    pub const CompareNe: AstExprBinary_Op = AstExprBinary_Op::CompareNe;
    pub const CompareEq: AstExprBinary_Op = AstExprBinary_Op::CompareEq;
    pub const CompareLt: AstExprBinary_Op = AstExprBinary_Op::CompareLt;
    pub const CompareLe: AstExprBinary_Op = AstExprBinary_Op::CompareLe;
    pub const CompareGt: AstExprBinary_Op = AstExprBinary_Op::CompareGt;
    pub const CompareGe: AstExprBinary_Op = AstExprBinary_Op::CompareGe;
    pub const And: AstExprBinary_Op = AstExprBinary_Op::And;
    pub const Or: AstExprBinary_Op = AstExprBinary_Op::Or;
    pub const Op__Count: AstExprBinary_Op = AstExprBinary_Op::Op__Count;
}

pub type AstExprBinaryOp = AstExprBinary_Op;
