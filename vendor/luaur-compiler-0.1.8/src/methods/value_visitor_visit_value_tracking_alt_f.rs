use crate::records::value_visitor::ValueVisitor;
use luaur_ast::records::ast_expr_function::AstExprFunction;

impl ValueVisitor {
    pub fn visit_ast_expr_function(&mut self, node: *mut AstExprFunction) -> bool {
        if node.is_null() {
            return false;
        }

        unsafe {
            let node_ref = &*node;
            let args = &node_ref.args;
            // C++ `variables[arg].init = nullptr`: operator[] overwrites `.init` for an
            // existing entry (try_insert would leave it unchanged).
            for i in 0..args.size {
                let arg = *args.data.add(i);
                if !arg.is_null() {
                    self.variables.get_or_insert(arg).init = core::ptr::null_mut();
                }
            }
        }

        true
    }
}
