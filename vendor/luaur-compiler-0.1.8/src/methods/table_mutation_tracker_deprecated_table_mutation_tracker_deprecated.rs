use crate::enums::table_constant_kind::TableConstantKind;
use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use crate::records::variable::Variable;
use luaur_ast::records::ast_local::AstLocal;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl<'a> TableMutationTrackerDeprecated<'a> {
    pub fn table_mutation_tracker_deprecated(
        constant_tables: &'a mut DenseHashMap<*mut AstLocal, TableConstantKind>,
        variables: &'a DenseHashMap<*mut AstLocal, Variable>,
    ) -> Self {
        TableMutationTrackerDeprecated {
            constant_tables,
            variables,
        }
    }
}
