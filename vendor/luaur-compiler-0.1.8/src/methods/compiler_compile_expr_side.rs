use crate::records::compiler::Compiler;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_expr_varargs::AstExprVarargs;
use luaur_ast::rtti;

impl Compiler {
    pub fn compile_expr_side(&mut self, node: *mut AstExpr) {
        unsafe {
            let ast_node = &*(node as *mut luaur_ast::records::ast_node::AstNode);
            if rtti::ast_node_is::<AstExprLocal>(ast_node)
                || rtti::ast_node_is::<AstExprGlobal>(ast_node)
                || rtti::ast_node_is::<AstExprVarargs>(ast_node)
                || rtti::ast_node_is::<AstExprFunction>(ast_node)
                || self.is_constant(node)
            {
                return;
            }

            if rtti::ast_node_as::<AstExprCall>(node as *mut luaur_ast::records::ast_node::AstNode)
                .is_null()
            {
                (*self.bytecode)
                    .add_debug_remark(format_args!("expression only compiled for side effects"));
            }

            let mut rsi = self.reg_scope_compiler();
            self.compile_expr_auto(node, &mut rsi);
        }
    }
}
