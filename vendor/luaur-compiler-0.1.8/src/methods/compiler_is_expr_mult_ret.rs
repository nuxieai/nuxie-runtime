use crate::functions::get_builtin_info::get_builtin_info;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_varargs::AstExprVarargs;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::enums::luau_builtin_function::LuauBuiltinFunction;

impl Compiler {
    pub fn is_expr_mult_ret(&mut self, node: *mut AstExpr) -> bool {
        unsafe {
            let expr = luaur_ast::rtti::ast_node_as::<AstExprCall>(node as *mut AstNode);
            if expr.is_null() {
                return luaur_ast::rtti::ast_node_as::<AstExprVarargs>(node as *mut AstNode)
                    .is_null()
                    == false;
            }

            if self.options.optimization_level <= 1 {
                return true;
            }

            if self.is_constant(expr as *mut AstExpr) {
                return false;
            }

            if self.options.optimization_level >= 2 {
                if let Some(bfid) = self.builtins.find(&expr) {
                    if *bfid != LuauBuiltinFunction::LBF_NONE as i32 {
                        return get_builtin_info(*bfid).results != 1;
                    }
                }
            }

            let func = self.get_function_expr((*expr).func);
            let fi = if func.is_null() {
                None
            } else {
                self.functions.find(&func)
            };

            if fi.map_or(false, |fi| fi.returns_one) {
                return false;
            }

            true
        }
    }
}
