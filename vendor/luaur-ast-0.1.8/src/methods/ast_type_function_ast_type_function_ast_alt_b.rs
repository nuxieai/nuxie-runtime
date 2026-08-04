use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_function::AstTypeFunction;
use crate::records::ast_type_list::AstTypeList;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;
use crate::type_aliases::ast_argument_name::AstArgumentName;
use luaur_common::LUAU_ASSERT;

impl AstTypeFunction {
    pub fn ast_type_function_location_ast_array_ast_attr_ast_array_ast_generic_type_ast_array_ast_generic_type_pack_ast_type_list_ast_array_optional_ast_argument_name_ast_type_pack(
        location: Location,
        attributes: AstArray<*mut AstAttr>,
        generics: AstArray<*mut AstGenericType>,
        generic_packs: AstArray<*mut AstGenericTypePack>,
        arg_types: AstTypeList,
        arg_names: AstArray<Option<AstArgumentName>>,
        return_types: *mut AstTypePack,
    ) -> Self {
        LUAU_ASSERT!(arg_names.len() == 0 || arg_names.len() == arg_types.types.len());

        Self {
            base: AstType {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            attributes,
            generics,
            generic_packs,
            arg_types,
            arg_names,
            return_types,
        }
    }
}
