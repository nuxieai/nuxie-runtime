//! `AstAttr::deprecatedInfo` (`Ast/src/Ast.cpp`).
//!
//! When the attribute is `@deprecated` and its first arg is a table literal,
//! pulls the `use`/`reason` string fields out of that table. `value` is a byte
//! array stored as `AstArray<char>`, so each string is rebuilt from the scalars'
//! low bytes.

use alloc::string::String;

use crate::records::ast_attr::{AstAttr, AstAttrType};
use crate::records::ast_expr_constant_string::AstExprConstantString;
use crate::records::ast_expr_table::AstExprTable;
use crate::records::ast_node::AstNode;
use crate::records::deprecated_info::DeprecatedInfo;

impl AstAttr {
    pub fn deprecated_info(&self) -> DeprecatedInfo {
        let mut info = DeprecatedInfo::default();
        info.deprecated = self.r#type == AstAttrType::Deprecated;

        if info.deprecated && self.args.len() > 0 {
            let arg0 = self.args.as_slice()[0];
            if !arg0.is_null()
                && crate::rtti::ast_node_is::<AstExprTable>(unsafe { &*(arg0 as *mut AstNode) })
            {
                let table =
                    unsafe { &*crate::rtti::ast_node_as::<AstExprTable>(arg0 as *mut AstNode) };
                info.use_ = string_field(table, b"use\0");
                info.reason = string_field(table, b"reason\0");
            }
        }

        info
    }
}

/// Looks up `key` in the table and, when its value is a string literal, returns
/// it as a `String` (rebuilt from the byte-valued `char` array).
fn string_field(table: &AstExprTable, key: &[u8]) -> Option<String> {
    let value = table.get_record(key.as_ptr() as *const core::ffi::c_char)?;
    if value.is_null()
        || !crate::rtti::ast_node_is::<AstExprConstantString>(unsafe { &*(value as *mut AstNode) })
    {
        return None;
    }
    let s = unsafe { &*crate::rtti::ast_node_as::<AstExprConstantString>(value as *mut AstNode) };
    Some(
        s.value
            .as_slice()
            .iter()
            .map(|&c| c as u8 as char)
            .collect(),
    )
}
