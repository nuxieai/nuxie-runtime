use crate::records::cost::Cost;
use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_stat_compound_assign::AstStatCompoundAssign;
use luaur_ast::records::ast_visitor::AstVisitor;
use luaur_ast::rtti::ast_node_is;

impl CostVisitor {
    pub fn visit_ast_stat_compound_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        if node.is_null() {
            return true;
        }

        unsafe {
            let node = &*(node as *mut AstStatCompoundAssign);

            // assign(node->var)
            self.assign(node.var);

            // if lhs is not a local, setting it requires an extra table operation
            let is_local = ast_node_is::<AstExprLocal>(
                &*(node.var as *mut luaur_ast::records::ast_node::AstNode),
            );
            let cost_increment = if is_local { 1 } else { 2 };
            self.result
                .operator_add_assign(&Cost::new(cost_increment, 0));
        }

        true
    }
}
