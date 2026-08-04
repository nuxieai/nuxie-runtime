use crate::functions::is_constant_false::is_constant_false;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn is_constant_false(&mut self, node: *mut AstExpr) -> bool {
        is_constant_false(&self.constants, node)
    }
}
