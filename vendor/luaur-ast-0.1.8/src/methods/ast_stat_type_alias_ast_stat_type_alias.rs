use crate::records::ast_array::AstArray;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_name::AstName;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_type_alias::AstStatTypeAlias;
use crate::records::ast_type::AstType;
use crate::records::location::Location;

impl AstStatTypeAlias {
    pub fn new_simple(
        location: Location,
        name: AstName,
        name_location: Location,
        generics: AstArray<*mut AstGenericType>,
        generic_packs: AstArray<*mut AstGenericTypePack>,
        type_: *mut AstType,
        exported: bool,
    ) -> Self {
        Self {
            base: AstStat {
                base: crate::records::ast_node::AstNode {
                    class_index: <Self as crate::rtti::AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            name,
            name_location,
            generics,
            generic_packs,
            type_ptr: type_,
            exported,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_type_alias_ast_stat_type_alias(
    location: Location,
    name: AstName,
    name_location: Location,
    generics: AstArray<*mut AstGenericType>,
    generic_packs: AstArray<*mut AstGenericTypePack>,
    type_: *mut AstType,
    exported: bool,
) -> AstStatTypeAlias {
    AstStatTypeAlias::new_simple(
        location,
        name,
        name_location,
        generics,
        generic_packs,
        type_,
        exported,
    )
}
