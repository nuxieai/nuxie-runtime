use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_stat_repeat::AstStatRepeat;

impl CostVisitor {
    pub fn visit_ast_stat_repeat(&mut self, node: *mut core::ffi::c_void) -> bool {
        unsafe {
            let node = &*(node as *mut AstStatRepeat);

            let condition = self.model(node.condition);

            // C++ `loop(node->body, condition)` uses the default factor of 3, not 1.
            self.loop_item(node.body, condition, 3);
        }

        false
    }
}
