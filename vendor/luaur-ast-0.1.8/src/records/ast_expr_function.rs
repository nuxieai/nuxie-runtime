//! Faithful port of Luau `AstExprFunction : AstExpr` (`Ast/include/Luau/Ast.h`).
//!
//! Hand-ported (false-blocked via the bare-name `AstAttr::Type` resolution).
//! `AstLocal* self` -> `self_` (Rust keyword). `std::optional<Location>` ->
//! `Option<Location>`. The two constructors and the
//! `has_native_attribute`/`has_attribute`/`get_attribute`/`visit` methods are
//! separate items.

use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_local::AstLocal;
use crate::records::ast_name::AstName;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::location::Location;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstExprFunction {
    pub base: AstExpr,
    pub attributes: AstArray<*mut AstAttr>,
    pub generics: AstArray<*mut AstGenericType>,
    pub generic_packs: AstArray<*mut AstGenericTypePack>,
    pub self_: *mut AstLocal,
    pub args: AstArray<*mut AstLocal>,
    pub return_annotation: *mut AstTypePack,
    pub vararg: bool,
    pub vararg_location: Location,
    pub vararg_annotation: *mut AstTypePack,
    pub body: *mut AstStatBlock,
    pub function_depth: usize,
    pub debugname: AstName,
    pub arg_location: Option<Location>,
}

impl crate::rtti::AstNodeClass for AstExprFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstExprFunction");
}
