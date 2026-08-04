use crate::records::value_visitor::ValueVisitor;
use luaur_ast::records::ast_stat_function::AstStatFunction;

impl ValueVisitor {
    pub fn visit_ast_stat_function(&mut self, node: *mut AstStatFunction) -> bool {
        unsafe {
            let node = &*node;
            self.assign(node.name);
            // C++ `node->func->visit(this)` is the AST node's dispatch — it runs the
            // visit_expr_function callback (registers args) AND recurses into the
            // function body. Calling the bare callback skipped the body, so no local
            // declared inside the function was ever tracked (recordValue then panicked).
            luaur_ast::visit::ast_expr_visit(
                node.func as *mut luaur_ast::records::ast_expr::AstExpr,
                self,
            );
        }
        false
    }
}
