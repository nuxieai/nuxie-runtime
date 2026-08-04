//! Faithful port of Luau `AstStatDeclareFunction : AstStat`
//! (`Ast/include/Luau/Ast.h`). Hand-ported (false-blocked via the bare-name
//! `AstAttr::Type` resolution). The two constructors and the
//! `visit`/`isCheckedFunction`/`has_attribute`/`get_attribute` methods are
//! separate items.

use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::location::Location;
use crate::type_aliases::ast_argument_name::AstArgumentName;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AstStatDeclareFunction {
    pub base: AstStat,
    pub attributes: AstArray<*mut AstAttr>,
    pub name: AstName,
    pub name_location: Location,
    pub generics: AstArray<*mut AstGenericType>,
    pub generic_packs: AstArray<*mut AstGenericTypePack>,
    pub params: AstTypeList,
    pub param_names: AstArray<AstArgumentName>,
    pub vararg: bool,
    pub vararg_location: Location,
    pub ret_types: *mut AstTypePack,
}

impl crate::rtti::AstNodeClass for AstStatDeclareFunction {
    const CLASS_INDEX: i32 = crate::rtti::ast_rtti_index("AstStatDeclareFunction");
}
