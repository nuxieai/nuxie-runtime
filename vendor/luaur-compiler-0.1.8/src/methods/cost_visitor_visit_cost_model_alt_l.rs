use crate::functions::always_terminates::always_terminates;
use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_stat_block::AstStatBlock;
use luaur_ast::visit::ast_stat_visit;

pub fn visit_ast_stat_block(this: &mut CostVisitor, node: *mut core::ffi::c_void) -> bool {
    unsafe {
        if node.is_null() {
            return false;
        }

        let block = &*(node as *const AstStatBlock);

        for i in 0..block.body.size {
            let stat = *block.body.data.add(i);

            ast_stat_visit(stat, this);

            // C++ stops modelling a block once a statement unconditionally terminates
            // (return/break/continue, or an if whose branches all terminate); the rest
            // is dead. The placeholder always-false here over-counted post-return code.
            if always_terminates(&*this.constants, stat) {
                break;
            }
        }
    }

    false
}

impl CostVisitor {
    pub fn visit_ast_stat_block(&mut self, node: *mut core::ffi::c_void) -> bool {
        visit_ast_stat_block(self, node)
    }
}
