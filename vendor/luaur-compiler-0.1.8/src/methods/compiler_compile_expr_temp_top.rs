use crate::records::compiler::Compiler;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn compile_expr_temp_top(&mut self, node: *mut AstExpr, target: u8) {
        let mut rs = self.reg_scope_compiler_i32(target as u32 + 1);
        self.compile_expr_temp(node, target);
    }
}
