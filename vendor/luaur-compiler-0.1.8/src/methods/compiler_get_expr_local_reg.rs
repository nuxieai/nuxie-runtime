use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn get_expr_local_reg(&mut self, node: *mut AstExpr) -> i32 {
        unsafe {
            let expr = self.get_expr_local(node);
            if !expr.is_null() {
                return match self.locals.find(&(*expr).local) {
                    Some(l) if l.allocated => l.reg as i32,
                    _ => -1,
                };
            }

            if luaur_common::FFlag::DebugLuauUserDefinedClasses.get() {
                let global = luaur_ast::rtti::ast_node_as::<
                    luaur_ast::records::ast_expr_global::AstExprGlobal,
                >(node as *mut _);
                if !global.is_null() {
                    if let Some(local) = self.class_locals.find(&(*global).name).copied() {
                        return self.get_local_reg(local);
                    }
                }
            }

            -1
        }
    }
}
