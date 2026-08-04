#[repr(C)]
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct AstExprTable {
    pub base: crate::records::ast_expr::AstExpr,
    pub items: crate::records::ast_array::AstArray<crate::records::ast_expr_table::Item>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub key: *mut crate::records::ast_expr::AstExpr,
    pub value: *mut crate::records::ast_expr::AstExpr,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    List = 0,
    Record = 1,
    General = 2,
}

impl crate::rtti::AstNodeClass for AstExprTable {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprTable");
}

pub mod Item_ {
    pub use super::ItemKind as Kind;
}
