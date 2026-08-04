use crate::enums::table_constant_kind::TableConstantKind;
use crate::functions::unwrap_expr_of_type::unwrap_expr_of_type;
use crate::records::table_mutation_tracker::TableMutationTracker;
use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use crate::records::variable::Variable;
use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::records::dense_hash_map::DenseHashMap;

pub fn build_table_constant_map(
    result: &mut DenseHashMap<*mut AstLocal, TableConstantKind>,
    variables: &DenseHashMap<*mut AstLocal, Variable>,
    root: *mut AstNode,
) {
    luaur_common::LUAU_ASSERT!(
        luaur_common::FFlag::LuauCompileFoldOptimize.get()
            && luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
    );

    if luaur_common::FFlag::LuauCompileNewTableMutationTracker.get() {
        let mut tracker = TableMutationTracker::table_mutation_tracker(variables);
        unsafe {
            luaur_ast::visit::ast_node_visit(root, &mut tracker);
        }

        for (local, var) in variables.iter() {
            if var.written {
                continue;
            }

            if var.init.is_null() || unwrap_expr_of_type::<AstExprTable>(var.init).is_null() {
                continue;
            }

            if !tracker.escaped.contains(local) {
                *result.get_or_insert(*local) = TableConstantKind::ConstantTable;
            }
        }
    } else {
        let mut mutation_tracker =
            TableMutationTrackerDeprecated::table_mutation_tracker_deprecated(result, variables);
        unsafe {
            luaur_ast::visit::ast_node_visit(root, &mut mutation_tracker);
        }
    }
}
