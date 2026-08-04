use crate::records::ast_array::AstArray;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_declare_function::AstStatDeclareFunction;
use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;
use crate::type_aliases::ast_argument_name::AstArgumentName;

impl AstStatDeclareFunction {
    pub fn new_simple(
        location: Location,
        name: AstName,
        name_location: Location,
        generics: AstArray<*mut AstGenericType>,
        generic_packs: AstArray<*mut AstGenericTypePack>,
        params: AstTypeList,
        param_names: AstArray<AstArgumentName>,
        vararg: bool,
        vararg_location: Location,
        ret_types: *mut AstTypePack,
    ) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            attributes: AstArray::default(),
            name,
            name_location,
            generics,
            generic_packs,
            params,
            param_names,
            vararg,
            vararg_location,
            ret_types,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_declare_function_location_ast_name_location_ast_array_ast_generic_type_ast_array_ast_generic_type_pack_ast_type_list_ast_array_ast_argument_name_bool_location_ast_type_pack(
    location: Location,
    name: AstName,
    name_location: Location,
    generics: AstArray<*mut AstGenericType>,
    generic_packs: AstArray<*mut AstGenericTypePack>,
    params: AstTypeList,
    param_names: AstArray<AstArgumentName>,
    vararg: bool,
    vararg_location: Location,
    ret_types: *mut AstTypePack,
) -> AstStatDeclareFunction {
    AstStatDeclareFunction::new_simple(
        location,
        name,
        name_location,
        generics,
        generic_packs,
        params,
        param_names,
        vararg,
        vararg_location,
        ret_types,
    )
}
