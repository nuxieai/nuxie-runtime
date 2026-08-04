use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn compile_expr_temp(&mut self, node: *mut AstExpr, target: u8) {
        self.compile_expr(node, target, true);
    }
}
