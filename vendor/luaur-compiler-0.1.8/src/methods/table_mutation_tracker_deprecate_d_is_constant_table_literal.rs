use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_group::AstExprGroup;
use luaur_ast::records::ast_expr_instantiate::AstExprInstantiate;
use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_expr_type_assertion::AstExprTypeAssertion;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti::ast_node_as;

impl TableMutationTrackerDeprecated<'_> {
    pub fn is_constant_table_literal(&self, node: *const AstExpr) -> bool {
        if node.is_null() {
            return false;
        }

        let node_ptr = node as *mut AstNode;

        let table = unsafe { ast_node_as::<AstExprTable>(node_ptr) };
        if !table.is_null() {
            for item in unsafe { (*table).items.iter() } {
                if !item.key.is_null() && !self.is_non_table_constant(item.key) {
                    return false;
                }
                if !self.is_non_table_constant(item.value) {
                    return false;
                }
            }
            return true;
        }

        let group = unsafe { ast_node_as::<AstExprGroup>(node_ptr) };
        if !group.is_null() {
            return self.is_constant_table_literal(unsafe { (*group).expr });
        }

        let assertion = unsafe { ast_node_as::<AstExprTypeAssertion>(node_ptr) };
        if !assertion.is_null() {
            return self.is_constant_table_literal(unsafe { (*assertion).expr });
        }

        let instantiate = unsafe { ast_node_as::<AstExprInstantiate>(node_ptr) };
        if !instantiate.is_null() {
            return self.is_constant_table_literal(unsafe { (*instantiate).expr });
        }

        false
    }
}

#[allow(non_snake_case)]
pub fn table_mutation_tracker_deprecate_d_is_constant_table_literal(
    tracker: &TableMutationTrackerDeprecated,
    node: *const AstExpr,
) -> bool {
    tracker.is_constant_table_literal(node)
}
