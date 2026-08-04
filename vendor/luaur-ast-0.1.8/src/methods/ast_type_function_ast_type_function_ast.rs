use crate::records::ast_array::AstArray;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_type::AstType;
use crate::records::ast_type_function::AstTypeFunction;
use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;
use crate::type_aliases::ast_argument_name::AstArgumentName;

impl AstTypeFunction {
    pub fn ast_type_function_location_ast_array_ast_generic_type_ast_array_ast_generic_type_pack_ast_type_list_ast_array_optional_ast_argument_name_ast_type_pack(
        location: Location,
        generics: AstArray<*mut AstGenericType>,
        generic_packs: AstArray<*mut AstGenericTypePack>,
        arg_types: AstTypeList,
        arg_names: AstArray<Option<AstArgumentName>>,
        return_types: *mut AstTypePack,
    ) -> Self {
        luaur_common::LUAU_ASSERT!(
            arg_names.len() == 0 || arg_names.len() == arg_types.types.len()
        );

        Self {
            base: AstType {
                base: crate::records::ast_node::AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            attributes: AstArray::default(),
            generics,
            generic_packs,
            arg_types,
            arg_names,
            return_types,
        }
    }
}
