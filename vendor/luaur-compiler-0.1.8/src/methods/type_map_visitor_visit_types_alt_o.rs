use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr_constant_bool::AstExprConstantBool;
use luaur_ast::records::ast_type::AstType;

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_constant_bool(&mut self, node: *mut AstExprConstantBool) -> bool {
        self.record_resolved_type_ast_expr_ast_type(
            node as *mut _,
            &self.builtin_types.boolean_type as *const _ as *const AstType,
        );

        false
    }
}
