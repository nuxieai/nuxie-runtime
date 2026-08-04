use crate::records::value_visitor::ValueVisitor;
use luaur_ast::records::ast_stat_local_function::AstStatLocalFunction;

impl ValueVisitor {
    pub fn visit_ast_stat_local_function(&mut self, node: *mut AstStatLocalFunction) -> bool {
        unsafe {
            let node_ref = &*node;
            self.variables.get_or_insert(node_ref.name as *mut _).init = node_ref.func as *mut _;
        }

        true
    }
}
