use crate::records::return_visitor::ReturnVisitor;
use luaur_ast::records::ast_expr::AstExpr;

impl ReturnVisitor {
    pub fn visit_ast_expr(&mut self, _expr: *mut AstExpr) -> bool {
        false
    }
}
