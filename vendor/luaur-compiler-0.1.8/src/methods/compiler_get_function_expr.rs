use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_expr_group::AstExprGroup;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_ast::records::ast_expr_instantiate::AstExprInstantiate;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_expr_type_assertion::AstExprTypeAssertion;
use luaur_ast::records::ast_node::AstNode;

impl Compiler {
    pub fn get_function_expr(&mut self, node: *mut AstExpr) -> *mut AstExprFunction {
        unsafe {
            let expr_local = luaur_ast::rtti::ast_node_as::<AstExprLocal>(node as *mut AstNode);
            if !expr_local.is_null() {
                let lv = self.variables.find(&(*expr_local).local);
                if lv.map_or(true, |lv| lv.written || lv.init.is_null()) {
                    return core::ptr::null_mut();
                }

                return self.get_function_expr(lv.unwrap().init);
            }

            let expr_index = luaur_ast::rtti::ast_node_as::<AstExprIndexName>(node as *mut AstNode);
            if !expr_index.is_null() && luaur_common::FFlag::LuauCompileInlineTableFunctions.get() {
                let value = self.try_index_constant_table(expr_index);
                if !value.is_null() {
                    return self.get_function_expr(value);
                }

                return core::ptr::null_mut();
            }

            let expr_group = luaur_ast::rtti::ast_node_as::<AstExprGroup>(node as *mut AstNode);
            if !expr_group.is_null() {
                return self.get_function_expr((*expr_group).expr);
            }

            let expr_assertion =
                luaur_ast::rtti::ast_node_as::<AstExprTypeAssertion>(node as *mut AstNode);
            if !expr_assertion.is_null() {
                return self.get_function_expr((*expr_assertion).expr);
            }

            let expr_instantiate =
                luaur_ast::rtti::ast_node_as::<AstExprInstantiate>(node as *mut AstNode);
            if !expr_instantiate.is_null()
                && luaur_common::FFlag::LuauCompileInlineTableFunctions.get()
            {
                return self.get_function_expr((*expr_instantiate).expr);
            }

            luaur_ast::rtti::ast_node_as::<AstExprFunction>(node as *mut AstNode)
        }
    }
}
