use crate::enums::table_constant_kind::TableConstantKind;
use crate::records::table_mutation_tracker_deprecated::TableMutationTrackerDeprecated;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_binary::{AstExprBinary, AstExprBinary_Op};
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
use luaur_ast::rtti::ast_node_as;
use luaur_ast::visit::AstVisitable;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl TableMutationTrackerDeprecated<'_> {
    pub fn observe_mutations(&mut self, node: *const AstExpr, could_mutate_table: bool) {
        if node.is_null() {
            return;
        }

        let node_ptr = node as *mut AstNode;

        if let Some(expr) = unsafe { ast_node_as::<AstExprGroup>(node_ptr).as_ref() } {
            self.observe_mutations(expr.expr, could_mutate_table);
        } else if !unsafe { ast_node_as::<AstExprConstantNil>(node_ptr) }.is_null()
            || !unsafe { ast_node_as::<AstExprConstantBool>(node_ptr) }.is_null()
            || !unsafe { ast_node_as::<AstExprConstantNumber>(node_ptr) }.is_null()
            || !unsafe { ast_node_as::<AstExprConstantInteger>(node_ptr) }.is_null()
            || !unsafe { ast_node_as::<AstExprConstantString>(node_ptr) }.is_null()
            || !unsafe { ast_node_as::<AstExprGlobal>(node_ptr) }.is_null()
            || !unsafe { ast_node_as::<AstExprVarargs>(node_ptr) }.is_null()
        {
            return;
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprLocal>(node_ptr).as_ref() } {
            let local = expr.local;
            if could_mutate_table {
                if let Some(kind) = self.constant_tables.find_mut(&local) {
                    *kind = TableConstantKind::NotConstant;
                }
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprCall>(node_ptr).as_ref() } {
            self.observe_mutations(expr.func, true);
            for arg in expr.args.as_slice() {
                self.observe_mutations(*arg, self.could_be_table_reference(*arg));
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprIndexName>(node_ptr).as_ref() } {
            self.observe_mutations(expr.expr, could_mutate_table);
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprIndexExpr>(node_ptr).as_ref() } {
            self.observe_mutations(expr.index, false);
            self.observe_mutations(expr.expr, could_mutate_table);
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprFunction>(node_ptr).as_ref() } {
            unsafe { (*expr.body).visit(self) };
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprTable>(node_ptr).as_ref() } {
            for item in expr.items.as_slice() {
                if !item.key.is_null() {
                    self.observe_mutations(item.key, false);
                }
                self.observe_mutations(item.value, self.could_be_table_reference(item.value));
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprUnary>(node_ptr).as_ref() } {
            self.observe_mutations(expr.expr, false);
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprBinary>(node_ptr).as_ref() } {
            let short_circuiting = expr.op == AstExprBinary_Op::And || expr.op == AstExprBinary_Op::Or;
            self.observe_mutations(expr.left, short_circuiting);
            self.observe_mutations(expr.right, short_circuiting);
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprTypeAssertion>(node_ptr).as_ref() } {
            self.observe_mutations(expr.expr, could_mutate_table);
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprIfElse>(node_ptr).as_ref() } {
            self.observe_mutations(expr.condition, false);
            self.observe_mutations(expr.true_expr, could_mutate_table);
            self.observe_mutations(expr.false_expr, could_mutate_table);
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprInterpString>(node_ptr).as_ref() } {
            for expression in expr.expressions.as_slice() {
                self.observe_mutations(*expression, false);
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprInstantiate>(node_ptr).as_ref() } {
            self.observe_mutations(expr.expr, could_mutate_table);
        } else {
            LUAU_ASSERT!(false);
        }
    }
}

#[allow(non_snake_case)]
pub fn table_mutation_tracker_deprecate_d_observe_mutations(
    tracker: &mut TableMutationTrackerDeprecated,
    node: *const AstExpr,
    could_mutate_table: bool,
) {
    tracker.observe_mutations(node, could_mutate_table)
}
