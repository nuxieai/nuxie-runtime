//! Node: `cxx:Method:Luau.Compiler:Compiler/src/ValueTracking.cpp:61:visit`
//!
//! `ValueVisitor::visit(AstStatCompoundAssign*)` — mark the compound-assignment
//! target written, then recurse into the value expression. Returns false: the
//! children are traversed here, not by the generic walker.

use crate::records::value_visitor::ValueVisitor;
use luaur_ast::records::ast_stat_compound_assign::AstStatCompoundAssign;

impl ValueVisitor {
    pub fn visit_ast_stat_compound_assign(&mut self, node: *mut AstStatCompoundAssign) -> bool {
        unsafe {
            let node = &*node;
            self.assign(node.var);
            luaur_ast::visit::ast_expr_visit(node.value, self);
        }

        false
    }
}
