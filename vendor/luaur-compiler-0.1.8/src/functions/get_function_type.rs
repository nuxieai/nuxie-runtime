use alloc::string::String;
use core::ffi::c_char;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_stat_type_alias::AstStatTypeAlias;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_bytecode_type::{
    LuauBytecodeType, LBC_TYPE_ANY, LBC_TYPE_FUNCTION, LBC_TYPE_TABLE,
};
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

use crate::functions::get_type::get_type;

pub(crate) fn get_function_type(
    func: *const AstExprFunction,
    type_aliases: &DenseHashMap<AstName, *mut AstStatTypeAlias>,
    host_vector_type: *const c_char,
    userdata_types: &DenseHashMap<AstName, u8>,
    bytecode: &mut BytecodeBuilder,
) -> String {
    let func_ref = unsafe { &*func };
    let self_ = !func_ref.self_.is_null();

    let mut type_info = String::new();
    let args_size = func_ref.args.as_slice().len();
    type_info.reserve(args_size + (self_ as usize) + 2);

    // C++ `std::string::push_back(uint8_t)` appends one raw byte. Rust `String::push(char)`
    // UTF-8-encodes the code point, so any byte >= 128 (every optional/userdata type, or a
    // function with >= 128 params) would expand to two bytes and corrupt the typeinfo. Push
    // raw bytes through `as_mut_vec()` to stay a faithful byte buffer.
    unsafe {
        type_info.as_mut_vec().push(LBC_TYPE_FUNCTION.0 as u8);
        type_info
            .as_mut_vec()
            .push(((self_ as usize) + args_size) as u8);

        if self_ {
            type_info.as_mut_vec().push(LBC_TYPE_TABLE.0 as u8);
        }
    }

    let mut have_non_any_param = false;
    for arg_ptr in func_ref.args.as_slice() {
        let arg = unsafe { &**arg_ptr };
        let mut seen_aliases = DenseHashSet::new(AstName::new());

        let ty = if !arg.annotation.is_null() {
            get_type(
                arg.annotation,
                func_ref.generics,
                type_aliases,
                true, // resolveAliases_DEPRECATED
                host_vector_type,
                userdata_types,
                bytecode,
                &mut seen_aliases,
            )
        } else {
            LBC_TYPE_ANY
        };

        if ty != LBC_TYPE_ANY {
            have_non_any_param = true;
        }

        unsafe { type_info.as_mut_vec().push(ty.0 as u8) };
    }

    // If all parameters simplify to any, we can just omit type info for this function
    if !have_non_any_param {
        return String::new();
    }

    type_info
}
