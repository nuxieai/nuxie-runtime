use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_stat_local::AstStatLocal;

use crate::functions::get_table_hint::get_table_hint;
use crate::records::shape_visitor::ShapeVisitor;

impl<'a> ShapeVisitor<'a> {
    pub fn visit_ast_stat_local(&mut self, node: *mut AstStatLocal) -> bool {
        // track local -> table association so that we can update table size prediction in assignField
        unsafe {
            if node.is_null() {
                return true;
            }

            let node_ref = &*node;

            if node_ref.vars.len() == 1 && node_ref.values.len() == 1 {
                let value_ptr = *node_ref.values.as_slice().get(0).unwrap_or(&core::ptr::null_mut());
                let table_ptr = get_table_hint(value_ptr);

                if !table_ptr.is_null() {
                    let table_ref: &AstExprTable = &*table_ptr;
                    if table_ref.items.len() == 0 {
                        let var_ptr = *node_ref.vars.as_slice().get(0).unwrap_or(&core::ptr::null_mut());
                        *self.tables.get_or_insert(var_ptr) = table_ptr;
                    }
                }
            }
        }

        true
    }
}
