use crate::records::return_visitor::ReturnVisitor;
use luaur_ast::records::ast_stat_return::AstStatReturn;

impl ReturnVisitor {
    pub fn visit_ast_stat_return(&mut self, stat: *mut AstStatReturn) -> bool {
        unsafe {
            self.returns_one &= (*stat).list.size == 1
                && !(*self.self_).is_expr_mult_ret(*(*stat).list.data.add(0));
        }
        false
    }
}
