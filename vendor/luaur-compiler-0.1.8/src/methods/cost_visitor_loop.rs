use crate::records::cost::Cost;
use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_stat_block::AstStatBlock;

impl CostVisitor {
    pub fn loop_item(&mut self, body: *mut AstStatBlock, iter_cost: Cost, factor: i32) {
        let before = self.result;

        self.result = Cost::default();

        unsafe {
            if !body.is_null() {
                self.visit_ast_stat_block(body as *mut core::ffi::c_void);
            }
        }

        self.result =
            before.operator_add(&self.result.operator_add(&iter_cost).operator_mul(factor));
    }
}

#[allow(dead_code)]
pub fn cost_visitor_loop(
    this: &mut CostVisitor,
    body: *mut AstStatBlock,
    iter_cost: Cost,
    factor: i32,
) {
    this.loop_item(body, iter_cost, factor);
}
