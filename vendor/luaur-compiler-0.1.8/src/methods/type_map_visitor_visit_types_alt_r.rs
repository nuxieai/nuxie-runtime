use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr_constant_string::AstExprConstantString;
use luaur_ast::records::ast_type::AstType;

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_constant_string(&mut self, node: *mut AstExprConstantString) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            self.record_resolved_type_ast_expr_ast_type(
                node as *mut _,
                &self.builtin_types.string_type as *const _ as *const AstType,
            );

            false
        }
    }
}
