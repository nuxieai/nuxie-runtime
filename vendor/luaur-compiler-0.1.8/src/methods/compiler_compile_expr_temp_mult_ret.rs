use crate::records::compiler::Compiler;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_varargs::AstExprVarargs;
use luaur_ast::rtti;

impl Compiler {
    pub fn compile_expr_temp_mult_ret(&mut self, node: *mut AstExpr, target: u8) -> bool {
        unsafe {
            let expr = rtti::ast_node_as::<AstExprCall>(node as *mut _);
            if !expr.is_null() {
                if self.options.optimization_level >= 2 && !self.is_expr_mult_ret(node) {
                    self.compile_expr_temp(node, target);
                    return false;
                }
                let mut rs = self.reg_scope_compiler_i32(target as u32);
                self.compile_expr_call(expr, target, 0, true, true);
                return true;
            } else {
                let expr = rtti::ast_node_as::<AstExprVarargs>(node as *mut _);
                if !expr.is_null() {
                    let mut rs = self.reg_scope_compiler_i32(target as u32);
                    self.compile_expr_varargs(expr, target, 0, true);
                    return true;
                } else {
                    self.compile_expr_temp(node, target);
                    return false;
                }
            }
        }
    }
}
