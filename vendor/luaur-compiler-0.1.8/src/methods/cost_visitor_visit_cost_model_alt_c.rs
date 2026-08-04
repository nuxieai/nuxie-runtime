use crate::records::cost::Cost;
use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_stat_for_in::AstStatForIn;

impl CostVisitor {
    pub fn visit_ast_stat_for_in(&mut self, node: *mut core::ffi::c_void) -> bool {
        unsafe {
            let node = &*(node as *mut AstStatForIn);

            for expr_ptr in node.values.as_slice() {
                let cost = self.model(*expr_ptr);
                // C++ `result += model(...)` — saturating add (Cost::operator+=).
                self.result.add_assign(&cost);
            }

            self.loop_item(
                node.body,
                Cost {
                    model: 1,
                    constant: 0,
                },
                3,
            );
        }

        false
    }
}
