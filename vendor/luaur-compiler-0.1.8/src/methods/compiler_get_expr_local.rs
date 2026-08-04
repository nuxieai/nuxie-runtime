use crate::functions::unwrap_expr_of_type::unwrap_expr_of_type;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_local::AstExprLocal;

impl Compiler {
    pub fn get_expr_local(&mut self, node: *mut AstExpr) -> *mut AstExprLocal {
        unwrap_expr_of_type::<AstExprLocal>(node)
    }
}
