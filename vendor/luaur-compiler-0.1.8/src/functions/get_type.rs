use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_generic_type::AstGenericType;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_stat_type_alias::AstStatTypeAlias;
use luaur_ast::records::ast_type::AstType;
use luaur_ast::records::ast_type_function::AstTypeFunction;
use luaur_ast::records::ast_type_group::AstTypeGroup;
use luaur_ast::records::ast_type_intersection::AstTypeIntersection;
use luaur_ast::records::ast_type_optional::AstTypeOptional;
use luaur_ast::records::ast_type_reference::AstTypeReference;
use luaur_ast::records::ast_type_singleton_bool::AstTypeSingletonBool;
use luaur_ast::records::ast_type_singleton_string::AstTypeSingletonString;
use luaur_ast::records::ast_type_table::AstTypeTable;
use luaur_ast::records::ast_type_union::AstTypeUnion;
use luaur_ast::rtti::{ast_node_as, ast_node_is};
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_bytecode_type::{
    LuauBytecodeType, LBC_TYPE_ANY, LBC_TYPE_BOOLEAN, LBC_TYPE_FUNCTION, LBC_TYPE_INVALID,
    LBC_TYPE_NIL, LBC_TYPE_NUMBER, LBC_TYPE_OPTIONAL_BIT, LBC_TYPE_STRING, LBC_TYPE_TABLE,
    LBC_TYPE_TAGGED_USERDATA_BASE, LBC_TYPE_USERDATA, LBC_TYPE_VECTOR,
};
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;
use luaur_common::FFlag;

pub fn get_type(
    ty: *const AstType,
    generics: AstArray<*mut AstGenericType>,
    type_aliases: &DenseHashMap<AstName, *mut AstStatTypeAlias>,
    resolve_aliases_deprecated: bool,
    host_vector_type: *const core::ffi::c_char,
    userdata_types: &DenseHashMap<AstName, u8>,
    bytecode: &mut BytecodeBuilder,
    seen_aliases: &mut DenseHashSet<AstName>,
) -> LuauBytecodeType {
    if ty.is_null() {
        return LBC_TYPE_ANY;
    }

    let node = ty as *mut luaur_ast::records::ast_node::AstNode;

    if let Some(ref_node) = unsafe { ast_node_as::<AstTypeReference>(node).as_ref() } {
        if ref_node.prefix.is_some() {
            return LBC_TYPE_ANY;
        }

        // C++ `if (alias && *alias)` — the entry may exist with a NULL value (a
        // block-scoped alias restored to its previous null binding on scope exit), in
        // which case it is NOT in scope and we must fall through to the generic/userdata
        // resolution. Only resolve when the stored pointer is non-null.
        if let Some(alias_ptr) = type_aliases
            .find(&ref_node.name)
            .copied()
            .filter(|p| !p.is_null())
        {
            let alias = unsafe { &*alias_ptr };
            if FFlag::LuauCompileTypeAliases.get() {
                if seen_aliases.contains(&alias.name) {
                    seen_aliases.clear();
                    return LBC_TYPE_ANY;
                } else {
                    seen_aliases.insert(ref_node.name);
                    return get_type(
                        alias.type_ptr,
                        alias.generics,
                        type_aliases,
                        /* resolveAliases_DEPRECATED= */ false,
                        host_vector_type,
                        userdata_types,
                        bytecode,
                        seen_aliases,
                    );
                }
            } else {
                // note: we only resolve aliases to the depth of 1 to avoid dealing with recursive aliases
                if resolve_aliases_deprecated {
                    return get_type(
                        alias.type_ptr,
                        alias.generics,
                        type_aliases,
                        /* resolveAliases_DEPRECATED= */ false,
                        host_vector_type,
                        userdata_types,
                        bytecode,
                        seen_aliases,
                    );
                } else {
                    return LBC_TYPE_ANY;
                }
            }
        }

        if crate::functions::is_generic::is_generic(ref_node.name, &generics) {
            return LBC_TYPE_ANY;
        }

        if !host_vector_type.is_null() && ref_node.name.operator_eq_c_char(host_vector_type) {
            return LBC_TYPE_VECTOR;
        }

        let prim = crate::functions::get_primitive_type::get_primitive_type(ref_node.name);
        if prim != LBC_TYPE_INVALID {
            return prim;
        }

        if let Some(userdata_index) = userdata_types.find(&ref_node.name) {
            bytecode.use_userdata_type(*userdata_index as u32);
            return LuauBytecodeType(LBC_TYPE_TAGGED_USERDATA_BASE.0 + *userdata_index as u16);
        }

        // not primitive or alias or generic => host-provided, we assume userdata for now
        return LBC_TYPE_USERDATA;
    } else if unsafe { ast_node_is::<AstTypeTable>(node) } {
        return LBC_TYPE_TABLE;
    } else if unsafe { ast_node_is::<AstTypeFunction>(node) } {
        return LBC_TYPE_FUNCTION;
    } else if let Some(un) = unsafe { ast_node_as::<AstTypeUnion>(node).as_ref() } {
        let mut optional = false;
        let mut r#type = LBC_TYPE_INVALID;

        for &ty in un.types.as_slice() {
            let et = get_type(
                ty,
                generics,
                type_aliases,
                resolve_aliases_deprecated,
                host_vector_type,
                userdata_types,
                bytecode,
                seen_aliases,
            );

            if et == LBC_TYPE_NIL {
                optional = true;
                continue;
            }

            if r#type == LBC_TYPE_INVALID {
                r#type = et;
                continue;
            }

            if r#type != et {
                return LBC_TYPE_ANY;
            }
        }

        if r#type == LBC_TYPE_INVALID {
            return LBC_TYPE_ANY;
        }

        return LuauBytecodeType(
            r#type.0
                | (if optional && (r#type != LBC_TYPE_ANY) {
                    LBC_TYPE_OPTIONAL_BIT.0
                } else {
                    0
                }),
        );
    } else if unsafe { ast_node_is::<AstTypeIntersection>(node) } {
        return LBC_TYPE_ANY;
    } else if let Some(group) = unsafe { ast_node_as::<AstTypeGroup>(node).as_ref() } {
        return get_type(
            group.type_,
            generics,
            type_aliases,
            resolve_aliases_deprecated,
            host_vector_type,
            userdata_types,
            bytecode,
            seen_aliases,
        );
    } else if unsafe { ast_node_is::<AstTypeOptional>(node) } {
        return LBC_TYPE_NIL;
    } else if unsafe { ast_node_is::<AstTypeSingletonBool>(node) } {
        return LBC_TYPE_BOOLEAN; // C++ returns LBC_TYPE_BOOLEAN for `true`/`false`
    } else if unsafe { ast_node_is::<AstTypeSingletonString>(node) } {
        return LBC_TYPE_STRING;
    }

    LBC_TYPE_ANY
}
