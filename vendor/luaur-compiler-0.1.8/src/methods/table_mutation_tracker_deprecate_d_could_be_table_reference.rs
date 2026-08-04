use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_binary::{AstExprBinary, AstExprBinary_Op};
use luaur_ast::records::ast_expr_group::AstExprGroup;
use luaur_ast::records::ast_expr_if_else::AstExprIfElse;
use luaur_ast::records::ast_expr_instantiate::AstExprInstantiate;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_expr_type_assertion::AstExprTypeAssertion;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti::{ast_node_as, ast_node_is};

impl TableMutationTrackerDeprecated<'_> {
    pub fn could_be_table_reference(&self, node: *mut AstExpr) -> bool {
        if node.is_null() {
            return false;
        }

        let node_ptr = node as *mut AstNode;

        let expr_group = unsafe { ast_node_as::<AstExprGroup>(node_ptr) };
        if !expr_group.is_null() {
            return self.could_be_table_reference(unsafe { (*expr_group).expr });
        }

        let expr_assertion = unsafe { ast_node_as::<AstExprTypeAssertion>(node_ptr) };
        if !expr_assertion.is_null() {
            return self.could_be_table_reference(unsafe { (*expr_assertion).expr });
        }

        let expr_instantiate = unsafe { ast_node_as::<AstExprInstantiate>(node_ptr) };
        if !expr_instantiate.is_null() {
            return self.could_be_table_reference(unsafe { (*expr_instantiate).expr });
        }

        let expr_if_else = unsafe { ast_node_as::<AstExprIfElse>(node_ptr) };
        if !expr_if_else.is_null() {
            return self.could_be_table_reference(unsafe { (*expr_if_else).true_expr })
                || self.could_be_table_reference(unsafe { (*expr_if_else).false_expr });
        }

        let bin_expr = unsafe { ast_node_as::<AstExprBinary>(node_ptr) };
        if !bin_expr.is_null() {
            let op = unsafe { (*bin_expr).op };
            if op == AstExprBinary_Op::And || op == AstExprBinary_Op::Or {
                return self.could_be_table_reference(unsafe { (*bin_expr).left })
                    || self.could_be_table_reference(unsafe { (*bin_expr).right });
            }
            return false;
        }

        if ast_node_is::<AstExprLocal>(unsafe { &*node_ptr }) {
            return true;
        }

        // We ignore AstExprIndexName and AstExprIndexExpr here since tables referencing other tables should be caught in the AstExprTable case
        // of observeMutations or the AstStatAssign visitor
        false
    }
}

#[allow(non_snake_case)]
pub fn table_mutation_tracker_deprecate_d_could_be_table_reference(
    tracker: &TableMutationTrackerDeprecated,
    node: *mut AstExpr,
) -> bool {
    tracker.could_be_table_reference(node)
}
