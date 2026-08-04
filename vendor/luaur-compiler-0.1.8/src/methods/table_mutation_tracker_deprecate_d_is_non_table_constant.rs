use crate::enums::table_constant_kind::TableConstantKind;
use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_binary::AstExprBinary;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_constant_bool::AstExprConstantBool;
use luaur_ast::records::ast_expr_constant_integer::AstExprConstantInteger;
use luaur_ast::records::ast_expr_constant_nil::AstExprConstantNil;
use luaur_ast::records::ast_expr_constant_number::AstExprConstantNumber;
use luaur_ast::records::ast_expr_constant_string::AstExprConstantString;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_expr_group::AstExprGroup;
use luaur_ast::records::ast_expr_if_else::AstExprIfElse;
use luaur_ast::records::ast_expr_index_expr::AstExprIndexExpr;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_ast::records::ast_expr_instantiate::AstExprInstantiate;
use luaur_ast::records::ast_expr_interp_string::AstExprInterpString;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_expr_type_assertion::AstExprTypeAssertion;
use luaur_ast::records::ast_expr_unary::AstExprUnary;
use luaur_ast::records::ast_expr_varargs::AstExprVarargs;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti::{ast_node_as, ast_node_is};
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl TableMutationTrackerDeprecated<'_> {
    pub fn is_non_table_constant(&self, node: *mut AstExpr) -> bool {
        if node.is_null() {
            return false;
        }

        let node_ptr = node as *mut AstNode;

        let expr_group = unsafe { ast_node_as::<AstExprGroup>(node_ptr) };
        if !expr_group.is_null() {
            return self.is_non_table_constant(unsafe { (*expr_group).expr });
        } else if ast_node_is::<AstExprConstantNil>(unsafe { &*node_ptr }) {
            return true;
        } else if ast_node_is::<AstExprConstantBool>(unsafe { &*node_ptr }) {
            return true;
        } else if ast_node_is::<AstExprConstantNumber>(unsafe { &*node_ptr }) {
            return true;
        } else if ast_node_is::<AstExprConstantInteger>(unsafe { &*node_ptr }) {
            return true;
        } else if ast_node_is::<AstExprConstantString>(unsafe { &*node_ptr }) {
            return true;
        }

        let expr_local = unsafe { ast_node_as::<AstExprLocal>(node_ptr) };
        if !expr_local.is_null() {
            let local_ptr = unsafe { (*expr_local).local };
            if let Some(kind) = self.constant_tables.find(&local_ptr) {
                return *kind == TableConstantKind::ConstantOther;
            } else {
                return false;
            }
        } else if ast_node_is::<AstExprGlobal>(unsafe { &*node_ptr }) {
            return false;
        } else if ast_node_is::<AstExprVarargs>(unsafe { &*node_ptr }) {
            return false;
        } else if ast_node_is::<AstExprCall>(unsafe { &*node_ptr }) {
            return false;
        }

        let expr_index_name = unsafe { ast_node_as::<AstExprIndexName>(node_ptr) };
        if !expr_index_name.is_null() {
            let local = unsafe { ast_node_as::<AstExprLocal>((*expr_index_name).expr as *mut AstNode) };
            if local.is_null() {
                return false;
            }

            let local_ptr = unsafe { (*local).local };
            if let Some(kind) = self.constant_tables.find(&local_ptr) {
                return *kind == TableConstantKind::ConstantTable;
            } else {
                return false;
            }
        }

        let expr_index_expr = unsafe { ast_node_as::<AstExprIndexExpr>(node_ptr) };
        if !expr_index_expr.is_null() {
            let local = unsafe { ast_node_as::<AstExprLocal>((*expr_index_expr).expr as *mut AstNode) };
            if local.is_null() {
                return false;
            }

            let local_ptr = unsafe { (*local).local };
            if let Some(kind) = self.constant_tables.find(&local_ptr) {
                return *kind == TableConstantKind::ConstantTable
                    && self.is_non_table_constant(unsafe { (*expr_index_expr).index });
            } else {
                return false;
            }
        } else if ast_node_is::<AstExprFunction>(unsafe { &*node_ptr }) {
            return false;
        } else if ast_node_is::<AstExprTable>(unsafe { &*node_ptr }) {
            return false;
        }

        let expr_unary = unsafe { ast_node_as::<AstExprUnary>(node_ptr) };
        if !expr_unary.is_null() {
            return self.is_non_table_constant(unsafe { (*expr_unary).expr });
        }

        let expr_binary = unsafe { ast_node_as::<AstExprBinary>(node_ptr) };
        if !expr_binary.is_null() {
            return self.is_non_table_constant(unsafe { (*expr_binary).left })
                && self.is_non_table_constant(unsafe { (*expr_binary).right });
        }

        let expr_assertion = unsafe { ast_node_as::<AstExprTypeAssertion>(node_ptr) };
        if !expr_assertion.is_null() {
            return self.is_non_table_constant(unsafe { (*expr_assertion).expr });
        }

        let expr_if_else = unsafe { ast_node_as::<AstExprIfElse>(node_ptr) };
        if !expr_if_else.is_null() {
            return self.is_non_table_constant(unsafe { (*expr_if_else).condition })
                && self.is_non_table_constant(unsafe { (*expr_if_else).true_expr })
                && self.is_non_table_constant(unsafe { (*expr_if_else).false_expr });
        }

        let expr_interp = unsafe { ast_node_as::<AstExprInterpString>(node_ptr) };
        if !expr_interp.is_null() {
            for expression in unsafe { (*expr_interp).expressions.iter() } {
                if !self.is_non_table_constant(*expression) {
                    return false;
                }
            }
            return true;
        }

        let expr_instantiate = unsafe { ast_node_as::<AstExprInstantiate>(node_ptr) };
        if !expr_instantiate.is_null() {
            return self.is_non_table_constant(unsafe { (*expr_instantiate).expr });
        } else {
            LUAU_ASSERT!(false);
        }

        false
    }
}

#[allow(non_snake_case)]
pub fn table_mutation_tracker_deprecate_d_is_non_table_constant(
    tracker: &TableMutationTrackerDeprecated,
    node: *mut AstExpr,
) -> bool {
    tracker.is_non_table_constant(node)
}
