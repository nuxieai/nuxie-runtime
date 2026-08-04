use crate::records::visitor::Visitor;
use luaur_ast::records::ast_expr_local::AstExprLocal;

impl Visitor {
    pub fn visit_ast_expr_local(&mut self, node: *mut AstExprLocal) -> bool {
        unsafe {
            let reg = (*self.self_).get_local_reg((*node).local);
            if reg >= 0 {
                let idx = (reg as usize) / 64;
                let bit = 1 << ((reg as usize) % 64);
                if (self.assigned[idx] & bit) != 0 {
                    self.conflict[idx] |= bit;
                }
            }
        }
        true
    }
}
