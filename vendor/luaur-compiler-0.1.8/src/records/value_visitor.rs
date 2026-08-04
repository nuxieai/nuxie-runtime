use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;

use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::global::Global;
use crate::records::variable::Variable;

// Data-structure pass: this lands the ValueVisitor STRUCT only. Its behavior
// (the `assign` method and the AstVisitor impl) are separate method nodes,
// translated in the ValueTracking module pass — the C++ `assign` writes through
// the `variables` map and dispatches via `var->visit(this)`, which the
// first-pass machine translation got wrong (it marked `lv->local` directly and
// invented `rtti::ast_*_visit` free functions). Deferred, not stubbed-to-lie.
#[derive(Debug, Clone)]
pub struct ValueVisitor {
    pub(crate) globals: DenseHashMap<AstName, Global>,
    pub(crate) variables: DenseHashMap<*mut AstLocal, Variable>,
}

// Wire the generic `AstVisitor` dispatch (used by `var->visit(this)` /
// `dispatch_node`) to the concrete `visit_ast_*` overloads. Only the statements
// the C++ `ValueVisitor` overrides are listed; everything else uses the trait's
// default (recurse) behavior.
impl luaur_ast::records::ast_visitor::AstVisitor for ValueVisitor {
    fn visit_stat_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_local(node as *mut luaur_ast::records::ast_stat_local::AstStatLocal)
    }

    fn visit_stat_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_assign(node as *mut luaur_ast::records::ast_stat_assign::AstStatAssign)
    }

    fn visit_stat_compound_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_compound_assign(
            node as *mut luaur_ast::records::ast_stat_compound_assign::AstStatCompoundAssign,
        )
    }

    fn visit_stat_local_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_local_function(
            node as *mut luaur_ast::records::ast_stat_local_function::AstStatLocalFunction,
        )
    }

    fn visit_stat_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_stat_function(
            node as *mut luaur_ast::records::ast_stat_function::AstStatFunction,
        )
    }

    fn visit_expr_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_function(
            node as *mut luaur_ast::records::ast_expr_function::AstExprFunction,
        )
    }
}
