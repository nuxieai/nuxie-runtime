use luaur_ast::records::ast_stat_assign::AstStatAssign;
use crate::records::shape_visitor::ShapeVisitor;

impl<'a> ShapeVisitor<'a> {
    pub fn visit_ast_stat_assign(&mut self, node: *mut AstStatAssign) -> bool {
        unsafe {
            let node = &*node;

            for i in 0..node.vars.len() {
                let var_ptr = *node.vars.as_slice().get(i).unwrap_or(&core::ptr::null_mut());
                self.assign(var_ptr);
            }

            for i in 0..node.values.len() {
                let val_ptr = *node.values.as_slice().get(i).unwrap_or(&core::ptr::null_mut());
                // In Luau AST, node->values.data[i]->visit(this) dispatches via ast_expr_visit
                luaur_ast::visit::ast_expr_visit(val_ptr, self);
            }
        }

        false
    }
}
