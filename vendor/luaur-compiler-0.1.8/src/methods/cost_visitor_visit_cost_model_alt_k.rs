use crate::records::cost::Cost;
use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_stat_continue::AstStatContinue;

impl CostVisitor {
    pub fn visit_ast_stat_continue(&mut self, node: *mut core::ffi::c_void) -> bool {
        let _node = node as *mut AstStatContinue;
        self.result.add_assign(&Cost::new(1, 0));

        false
    }
}
