use alloc::vec::Vec;
use core::ffi::c_char;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_interp_string::AstExprInterpString;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;

pub fn fold_interp_string(
    result: &mut Constant,
    expr: *mut AstExprInterpString,
    constants: &mut DenseHashMap<*mut AstExpr, Constant>,
    string_table: &mut AstNameTable,
) {
    let expr = unsafe { &*expr };
    LUAU_ASSERT!(expr.strings.len() == expr.expressions.len() + 1);

    let mut result_length: usize = 0;
    for index in 0..expr.strings.len() {
        let string = expr.strings.as_slice()[index];
        result_length += string.len();
        if index < expr.expressions.len() {
            let expr_ptr = expr.expressions.as_slice()[index];
            let c = constants.find(&expr_ptr);
            LUAU_ASSERT!(c.is_some());
            let c = c.unwrap();
            LUAU_ASSERT!(c.r#type == Type::Type_String);
            result_length += c.string_length as usize;
        }
    }

    const K_CONSTANT_FOLD_STRING_LIMIT: usize = 4096;
    if result_length > K_CONSTANT_FOLD_STRING_LIMIT {
        return;
    }

    result.r#type = Type::Type_String;
    result.string_length = result_length as u32;

    if result_length == 0 {
        // C++ `result.valueString = ""` — a non-null pointer to a static empty C-string.
        // A null here later trips sref()'s `LUAU_ASSERT(data.begin())` when the folded
        // empty interpolation (e.g. `{empty}`) is emitted as a string constant.
        unsafe {
            result.data.value_string = c"".as_ptr();
        }
        return;
    }

    let mut tmp = Vec::with_capacity(result_length);

    for index in 0..expr.strings.len() {
        let string = expr.strings.as_slice()[index];
        let slice = unsafe {
            core::slice::from_raw_parts(string.as_slice().as_ptr() as *const u8, string.len())
        };
        tmp.extend_from_slice(slice);
        if index < expr.expressions.len() {
            let expr_ptr = expr.expressions.as_slice()[index];
            let c = constants.find(&expr_ptr);
            LUAU_ASSERT!(c.is_some());
            let c = c.unwrap();
            let string_slice = c.get_string();
            let string_bytes = unsafe {
                core::slice::from_raw_parts(
                    string_slice.as_slice().as_ptr() as *const u8,
                    string_slice.len(),
                )
            };
            tmp.extend_from_slice(string_bytes);
        }
    }

    result.r#type = Type::Type_String;
    result.string_length = result_length as u32;
    let name = string_table.get_or_add(tmp.as_ptr() as *const c_char, tmp.len());
    unsafe {
        result.data.value_string = name.value;
    }
}
