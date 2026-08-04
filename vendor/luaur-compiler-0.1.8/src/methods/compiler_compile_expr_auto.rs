use crate::records::compiler::Compiler;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn compile_expr_auto(&mut self, node: *mut AstExpr, _rs: &mut RegScope) -> u8 {
        let reg = self.get_expr_local_reg(node);
        if reg >= 0 {
            return reg as u8;
        }

        let reg = self.alloc_reg(node as *mut _, 1);
        self.compile_expr_temp(node, reg);
        reg
    }
}
