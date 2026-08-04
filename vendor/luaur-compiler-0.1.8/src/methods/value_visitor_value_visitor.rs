use crate::enums::global::Global;
use crate::records::value_visitor::ValueVisitor;
use crate::records::variable::Variable;
use core::mem;
use core::ptr;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl ValueVisitor {
    pub fn value_visitor(
        globals: &mut DenseHashMap<AstName, Global>,
        variables: &mut DenseHashMap<*mut AstLocal, Variable>,
    ) -> Self {
        let globals_owned = mem::replace(globals, DenseHashMap::new(AstName::new()));
        let variables_owned = mem::replace(variables, DenseHashMap::new(ptr::null_mut()));
        ValueVisitor {
            globals: globals_owned,
            variables: variables_owned,
        }
    }
}
