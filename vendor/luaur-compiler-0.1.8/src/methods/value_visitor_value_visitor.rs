use crate::enums::global::Global;
use crate::records::value_visitor::ValueVisitor;
use crate::records::variable::Variable;
use core::mem;
use core::ptr;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

impl ValueVisitor {
    pub fn value_visitor(
        globals: &mut DenseHashMap<AstName, Global>,
        variables: &mut DenseHashMap<*mut AstLocal, Variable>,
        class_locals: &mut DenseHashMap<AstName, *mut AstLocal>,
    ) -> Self {
        let globals_owned = mem::replace(globals, DenseHashMap::new(AstName::new()));
        let variables_owned = mem::replace(variables, DenseHashMap::new(ptr::null_mut()));
        let class_locals_owned = mem::replace(class_locals, DenseHashMap::new(AstName::new()));
        ValueVisitor {
            globals: globals_owned,
            variables: variables_owned,
            class_locals: class_locals_owned,
            exported_functions: core::ptr::null_mut(),
            exported_variables: core::ptr::null_mut(),
        }
    }

    pub fn value_visitor_with_exports(
        globals: &mut DenseHashMap<AstName, Global>,
        variables: &mut DenseHashMap<*mut AstLocal, Variable>,
        class_locals: &mut DenseHashMap<AstName, *mut AstLocal>,
        exported_functions: &mut DenseHashSet<*mut AstLocal>,
        exported_variables: &mut alloc::vec::Vec<*mut AstLocal>,
    ) -> Self {
        let mut visitor = Self::value_visitor(globals, variables, class_locals);
        visitor.exported_functions = exported_functions;
        visitor.exported_variables = exported_variables;
        visitor
    }
}
