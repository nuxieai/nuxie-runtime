use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_name::AstName;

use luaur_common::records::dense_hash_pointer::DenseHashPointer;

use std::hash::{Hash, Hasher};

pub fn shape_visitor_hasher_operator_call(p: (*mut AstExprTable, AstName)) -> usize {
    let mut state = std::collections::hash_map::DefaultHasher::new();
    p.1.hash(&mut state);
    let name_hash = state.finish() as usize;
    DenseHashPointer.call(p.0 as *const core::ffi::c_void) ^ name_hash
}
