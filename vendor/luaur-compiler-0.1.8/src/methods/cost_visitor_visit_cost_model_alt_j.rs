use crate::records::cost::Cost;
use crate::records::cost_visitor::CostVisitor;

impl CostVisitor {
    pub fn visit_ast_stat_break(&mut self, _node: *mut core::ffi::c_void) -> bool {
        self.result.add_assign(&Cost::new(1, 0));
        false
    }
}
