use crate::records::compiler::Compiler;
use crate::records::l_value::LValue;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn compile_assign(&mut self, lv: &LValue, source: u8, target_expr: *mut AstExpr) {
        self.compile_l_value_use(lv, source, true, target_expr);
    }
}
