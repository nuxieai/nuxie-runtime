//! `AstExprTable::getRecord` (`Ast/src/Ast.cpp:396`).
//!
//! Returns the value expr of the `Record` item whose key string equals `key`
//! (a nul-terminated C string). C++: `strcmp(item.key->as<AstExprConstantString>
//! ()->value.data, key) == 0`. `value` is a `char` (byte) array; the Rust record
//! models it as `AstArray<char>`, so we compare each scalar's low byte against
//! `key`'s bytes up to its nul terminator.

use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_constant_string::AstExprConstantString;
use crate::records::ast_expr_table::{AstExprTable, ItemKind};
use crate::records::ast_node::AstNode;

impl AstExprTable {
    pub fn get_record(&self, key: *const core::ffi::c_char) -> Option<*mut AstExpr> {
        for item in self.items.iter() {
            if item.kind == ItemKind::Record {
                let string_expr = unsafe {
                    crate::rtti::ast_node_as::<AstExprConstantString>(item.key as *mut AstNode)
                };
                if !string_expr.is_null() {
                    let value = unsafe { (*string_expr).value };
                    if char_array_eq_cstr(value.as_slice(), key) {
                        return Some(item.value);
                    }
                }
            }
        }
        None
    }
}

/// `strcmp(value.data, key) == 0` where `value` is a (non-nul-terminated) byte
/// array stored as `char`s and `key` is a nul-terminated C string.
fn char_array_eq_cstr(value: &[core::ffi::c_char], key: *const core::ffi::c_char) -> bool {
    let mut i = 0usize;
    loop {
        let k = unsafe { *key.add(i) } as u8;
        if k == 0 {
            return i == value.len();
        }
        if i >= value.len() || (value[i] as u32) != k as u32 {
            return false;
        }
        i += 1;
    }
}
