use crate::records::compiler::Compiler;
use crate::records::l_value::LValue;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_stat_function::AstStatFunction;

impl Compiler {
    pub fn compile_stat_function(&mut self, stat: *mut AstStatFunction) {
        unsafe {
            let stat_ref = &*stat;
            if let reg = self.get_expr_local_reg(stat_ref.name) {
                if reg >= 0 {
                    self.compile_expr(
                        stat_ref.func as *mut luaur_ast::records::ast_expr::AstExpr,
                        reg as u8,
                        false,
                    );
                    return;
                }
            }

            let mut rs = self.reg_scope_compiler();
            let reg = self.alloc_reg(stat as *mut _, 1);
            self.compile_expr_temp(
                stat_ref.func as *mut luaur_ast::records::ast_expr::AstExpr,
                reg,
            );
            let var = self.compile_l_value(stat_ref.name, &mut rs);
            self.compile_assign(&var, reg, stat_ref.name);
        }
    }
}
