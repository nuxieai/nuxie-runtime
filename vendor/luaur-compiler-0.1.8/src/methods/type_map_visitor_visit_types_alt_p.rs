use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_constant_number::AstExprConstantNumber;
use luaur_ast::records::ast_type::AstType;

impl<'a> TypeMapVisitor<'a> {
    pub fn visit_ast_expr_constant_number(&mut self, node: *mut AstExprConstantNumber) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            // builtin_types.number_type is an AstTypeReference (or similar wrapper).
            // The record_resolved_type_ast_expr_ast_type method expects *const AstType.
            // In Luau AST, AstTypeReference contains a base AstType at offset 0.
            let ty_ptr = &self.builtin_types.number_type as *const _ as *const AstType;

            self.record_resolved_type_ast_expr_ast_type(node as *mut AstExpr, ty_ptr);

            false
        }
    }
}
