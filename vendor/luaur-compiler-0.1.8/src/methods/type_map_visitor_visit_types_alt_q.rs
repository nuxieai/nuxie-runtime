use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_constant_integer::AstExprConstantInteger;
use luaur_ast::records::ast_type::AstType;

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_constant_integer(&mut self, node: *mut AstExprConstantInteger) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            // Corresponds to: recordResolvedType(node, &builtinTypes.integerType);
            // builtin_types.integer_type is an AstTypeReference; we need to cast its address to the base AstType pointer.
            let integer_ty = &self.builtin_types.integer_type as *const _ as *const AstType;

            self.record_resolved_type_ast_expr_ast_type(node as *mut AstExpr, integer_ty);

            false
        }
    }
}
