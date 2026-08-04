//! Faithful port of Luau `AstAttr : AstNode` (`Ast/include/Luau/Ast.h`).
//!
//! Hand-ported (the scheduler false-blocks it: its nested `Type` enum resolves
//! by bare name to an unrelated `Type` in `Lexer.h`). The nested
//! `enum Type { Checked, Native, Deprecated, DebugNoinline, Unknown }` is
//! inlined here as `AstAttrType` (matching how the other nested AST enums live
//! in their owner's record file). `DeprecatedInfo` is its own already-translated
//! record; the `deprecatedInfo()`/`visit`/`as_attr` methods are separate items.

use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;

/// C++ `AstAttr::Type`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AstAttrType {
    Checked,
    Native,
    Deprecated,
    DebugNoinline,
    Unknown,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstAttr {
    pub base: AstNode,
    pub r#type: AstAttrType,
    pub args: AstArray<*mut AstExpr>,
    pub name: AstName,
}

impl crate::rtti::AstNodeClass for AstAttr {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstAttr");
}
