use crate::records::type_map_visitor::TypeMapVisitor;

use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_table_indexer::AstTableIndexer;
use luaur_ast::records::ast_type_table::AstTypeTable;
use luaur_ast::rtti;

impl<'a> TypeMapVisitor<'a> {
    pub fn try_get_table_indexer(&self, expr: *mut AstExpr) -> *mut AstTableIndexer {
        unsafe {
            if let Some(type_ptr) = self.resolved_exprs.find(&expr) {
                if !(*type_ptr).is_null() {
                    let node_ptr = *type_ptr as *mut AstNode;
                    let table_ty = rtti::ast_node_as::<AstTypeTable>(node_ptr);
                    if !table_ty.is_null() {
                        return (*table_ty).indexer;
                    }
                }
            }
        }
        core::ptr::null_mut()
    }
}
