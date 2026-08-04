use crate::functions::unwrap_expr_of_type::unwrap_expr_of_type;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_group::AstExprGroup;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_expr_type_assertion::AstExprTypeAssertion;
use luaur_ast::records::ast_node::AstNode;

impl Compiler {
    pub fn get_expr_local(&mut self, node: *mut AstExpr) -> *mut AstExprLocal {
        unsafe {
            if luaur_common::FFlag::LuauCompileInlineTableFunctions.get() {
                return unwrap_expr_of_type::<AstExprLocal>(node);
            }

            let expr = luaur_ast::rtti::ast_node_as::<AstExprLocal>(node as *mut AstNode);
            if !expr.is_null() {
                return expr;
            }

            let group = luaur_ast::rtti::ast_node_as::<AstExprGroup>(node as *mut AstNode);
            if !group.is_null() {
                return self.get_expr_local((*group).expr);
            }

            let assertion =
                luaur_ast::rtti::ast_node_as::<AstExprTypeAssertion>(node as *mut AstNode);
            if !assertion.is_null() {
                return self.get_expr_local((*assertion).expr);
            }

            core::ptr::null_mut()
        }
    }
}
