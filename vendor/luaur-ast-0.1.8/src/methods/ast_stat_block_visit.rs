use crate::records::ast_stat_block::AstStatBlock;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstStatBlock {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_stat_block(self as *const Self as *mut core::ffi::c_void) {
            for stat_ptr in self.body.iter() {
                unsafe {
                    crate::visit::ast_stat_visit(*stat_ptr, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_block_visit(this: &AstStatBlock, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
