//! Faithful port of Luau `AstTypeFunction : AstType` (`Ast/include/Luau/Ast.h`).
//! Hand-ported (false-blocked via the bare-name `AstAttr::Type` resolution).
//! `AstArray<std::optional<AstArgumentName>>` -> `AstArray<Option<AstArgumentName>>`.
//! The two constructors and the `visit`/`isCheckedFunction`/`has_attribute`/
//! `get_attribute` methods are separate items.

use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_type::AstType;
use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack::AstTypePack;
use crate::type_aliases::ast_argument_name::AstArgumentName;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstTypeFunction {
    pub base: AstType,
    pub attributes: AstArray<*mut AstAttr>,
    pub generics: AstArray<*mut AstGenericType>,
    pub generic_packs: AstArray<*mut AstGenericTypePack>,
    pub arg_types: AstTypeList,
    pub arg_names: AstArray<Option<AstArgumentName>>,
    pub return_types: *mut AstTypePack,
}

impl crate::rtti::AstNodeClass for AstTypeFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstTypeFunction");
}
