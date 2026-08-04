use crate::records::undefined_local_visitor::UndefinedLocalVisitor;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl UndefinedLocalVisitor {
    pub fn visit_ast_expr_function(&mut self, node: *mut AstExprFunction) -> bool {
        unsafe {
            let f = (*self.self_).functions.find(&node);
            LUAU_ASSERT!(f.is_some());
            let f = f.unwrap();

            for uv in &f.upvals {
                LUAU_ASSERT!((*(*uv)).function_depth < (*node).function_depth);

                if (*(*uv)).function_depth == (*node).function_depth - 1 {
                    self.check(*uv);
                }
            }
        }
        false
    }
}
