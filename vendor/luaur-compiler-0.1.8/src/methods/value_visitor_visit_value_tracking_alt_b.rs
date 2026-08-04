//! Node: `cxx:Method:Luau.Compiler:Compiler/src/ValueTracking.cpp:50:visit`
//!
//! `ValueVisitor::visit(AstStatAssign*)` — mark every assignment target written,
//! then recurse into the value expressions (so nested assignments like
//! `t[function() t = nil end] = 5` are tracked). Returns false: traversal of the
//! children is performed here, not by the generic walker.

use crate::records::value_visitor::ValueVisitor;
use luaur_ast::records::ast_stat_assign::AstStatAssign;

impl ValueVisitor {
    pub fn visit_ast_stat_assign(&mut self, node: *mut AstStatAssign) -> bool {
        unsafe {
            let node = &*node;
            for i in 0..node.vars.size {
                self.assign(*node.vars.data.add(i));
            }
            for i in 0..node.values.size {
                luaur_ast::visit::ast_expr_visit(*node.values.data.add(i), self);
            }
        }

        false
    }
}
