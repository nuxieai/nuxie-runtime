//! Node: `cxx:Function:Luau.Compiler:Compiler/src/ValueTracking.cpp:104:trackValues`
//!
//! `trackValues` — run a `ValueVisitor` over the AST root to record, for every
//! local, its initializer and whether it is ever reassigned (and which globals
//! are written). The C++ visitor holds references to `globals`/`variables`; the
//! Rust `ValueVisitor` owns them, so the constructor moves the maps in and we
//! move the populated results back out after the walk.

use crate::enums::global::Global;
use crate::records::value_visitor::ValueVisitor;
use crate::records::variable::Variable;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

pub fn track_values(
    globals: &mut DenseHashMap<AstName, Global>,
    variables: &mut DenseHashMap<*mut AstLocal, Variable>,
    class_locals: &mut DenseHashMap<AstName, *mut AstLocal>,
    exported_functions: &mut DenseHashSet<*mut AstLocal>,
    exported_variables: &mut alloc::vec::Vec<*mut AstLocal>,
    root: *mut AstNode,
) {
    let mut visitor = ValueVisitor::value_visitor_with_exports(
        globals,
        variables,
        class_locals,
        exported_functions,
        exported_variables,
    );

    unsafe {
        luaur_ast::visit::dispatch_node(root, &mut visitor);
    }

    *globals = visitor.globals;
    *variables = visitor.variables;
    *class_locals = visitor.class_locals;
}

pub fn track_values_deprecated(
    globals: &mut DenseHashMap<AstName, Global>,
    variables: &mut DenseHashMap<*mut AstLocal, Variable>,
    class_locals: &mut DenseHashMap<AstName, *mut AstLocal>,
    root: *mut AstNode,
) {
    let mut visitor = ValueVisitor::value_visitor(globals, variables, class_locals);

    unsafe {
        luaur_ast::visit::dispatch_node(root, &mut visitor);
    }

    *globals = visitor.globals;
    *variables = visitor.variables;
    *class_locals = visitor.class_locals;
}
