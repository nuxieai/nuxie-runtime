use crate::functions::is_constant_true::is_constant_true;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn is_constant_true(&mut self, node: *mut AstExpr) -> bool {
        is_constant_true(&self.constants, node)
    }
}
