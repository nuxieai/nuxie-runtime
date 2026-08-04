use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::global::Global;
use crate::records::builtin_visitor::BuiltinVisitor;
use crate::records::compile_options::CompileOptions;
use crate::records::variable::Variable;

pub fn analyze_builtins(
    result: &mut DenseHashMap<*mut AstExprCall, i32>,
    globals: &DenseHashMap<AstName, Global>,
    variables: &DenseHashMap<*mut AstLocal, Variable>,
    options: &CompileOptions,
    root: *mut AstNode,
    names: &AstNameTable,
) {
    let mut visitor = BuiltinVisitor::new(result, globals, variables, options, names);
    // C++ `root->visit(&visitor)` — traverse the WHOLE tree so the visitor's
    // visit_expr_call fires for every call. The model instead cast `root` (an
    // AstStatBlock!) to AstExprCall and called `visit` once, registering nothing
    // -> all optimization-level-2 builtin constant-folding silently no-op'd.
    unsafe {
        luaur_ast::visit::ast_node_visit(root, &mut visitor);
    }
}
