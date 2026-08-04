use crate::records::constant_visitor::ConstantVisitor;
use luaur_ast::records::ast_expr::AstExpr;

impl<'a> ConstantVisitor<'a> {
    pub fn visit_ast_expr(&mut self, node: *mut AstExpr) -> bool {
        self.analyze(node);
        false
    }
}
