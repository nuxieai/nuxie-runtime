use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_binary::AstExprBinary;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_group::AstExprGroup;
use luaur_ast::records::ast_expr_if_else::AstExprIfElse;
use luaur_ast::records::ast_expr_index_expr::AstExprIndexExpr;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_ast::records::ast_expr_instantiate::AstExprInstantiate;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_expr_type_assertion::AstExprTypeAssertion;
use luaur_ast::records::ast_stat_assign::AstStatAssign;
use luaur_ast::records::ast_stat_compound_assign::AstStatCompoundAssign;
use luaur_ast::records::ast_stat_for_in::AstStatForIn;
use luaur_ast::records::ast_stat_function::AstStatFunction;
use luaur_ast::records::ast_stat_local::AstStatLocal;
use luaur_ast::records::ast_stat_return::AstStatReturn;
use luaur_ast::records::ast_visitor::AstVisitor;

use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

use crate::records::variable::Variable;
use luaur_ast::records::ast_local::AstLocal;

#[derive(Debug)]
pub struct TableMutationTracker<'a> {
    pub(crate) variables: &'a DenseHashMap<*mut AstLocal, Variable>,
    pub(crate) escaped: DenseHashSet<*mut AstLocal>,
}

impl<'a> TableMutationTracker<'a> {
    pub fn table_mutation_tracker(variables: &'a DenseHashMap<*mut AstLocal, Variable>) -> Self {
        Self {
            variables,
            escaped: DenseHashSet::new(core::ptr::null_mut()),
        }
    }

    pub fn mark_escaped(&mut self, mut expr: *mut AstExpr) {
        loop {
            if expr.is_null() {
                return;
            }
            unsafe {
                let node_ptr = expr as *mut luaur_ast::records::ast_node::AstNode;

                let local = luaur_ast::rtti::ast_node_as::<AstExprLocal>(node_ptr);
                if !local.is_null() {
                    self.escaped.insert((*local).local);
                    return;
                }

                let group = luaur_ast::rtti::ast_node_as::<AstExprGroup>(node_ptr);
                if !group.is_null() {
                    expr = (*group).expr;
                    continue;
                }

                let assertion = luaur_ast::rtti::ast_node_as::<AstExprTypeAssertion>(node_ptr);
                if !assertion.is_null() {
                    expr = (*assertion).expr;
                    continue;
                }

                let inst = luaur_ast::rtti::ast_node_as::<AstExprInstantiate>(node_ptr);
                if !inst.is_null() {
                    expr = (*inst).expr;
                    continue;
                }

                let if_else = luaur_ast::rtti::ast_node_as::<AstExprIfElse>(node_ptr);
                if !if_else.is_null() {
                    self.mark_escaped((*if_else).true_expr);
                    expr = (*if_else).false_expr;
                    continue;
                }

                let bin = luaur_ast::rtti::ast_node_as::<AstExprBinary>(node_ptr);
                if !bin.is_null() {
                    if (*bin).op == AstExprBinary::And || (*bin).op == AstExprBinary::Or {
                        self.mark_escaped((*bin).left);
                        expr = (*bin).right;
                        continue;
                    } else {
                        return;
                    }
                }

                return;
            }
        }
    }

    pub fn mark_escaped_table_index(&mut self, expr: *mut AstExpr, is_lvalue: bool) {
        if expr.is_null() {
            return;
        }
        unsafe {
            let node_ptr = expr as *mut luaur_ast::records::ast_node::AstNode;

            let idx_name = luaur_ast::rtti::ast_node_as::<AstExprIndexName>(node_ptr);
            if !idx_name.is_null() {
                self.mark_escaped((*idx_name).expr);
                return;
            }

            let idx_expr = luaur_ast::rtti::ast_node_as::<AstExprIndexExpr>(node_ptr);
            if !idx_expr.is_null() {
                self.mark_escaped((*idx_expr).expr);
                if is_lvalue {
                    self.mark_escaped((*idx_expr).index);
                }
            }
        }
    }
}

impl<'a> AstVisitor for TableMutationTracker<'a> {
    fn visit_expr_call(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = node as *mut AstExprCall;
        unsafe {
            for arg in (*node).args.iter() {
                self.mark_escaped(*arg);
            }

            if (*node).self_ {
                self.mark_escaped_table_index((*node).func, false);
            }
        }

        true
    }

    fn visit_expr_table(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = node as *mut AstExprTable;
        unsafe {
            for item in (*node).items.iter() {
                if !item.key.is_null() {
                    self.mark_escaped(item.key);
                }
                self.mark_escaped(item.value);
            }
        }
        true
    }

    fn visit_stat_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = node as *mut AstStatLocal;
        unsafe {
            for (value, _var) in (*node).values.iter().zip((*node).vars.iter()) {
                self.mark_escaped(*value);
            }
        }
        true
    }

    fn visit_stat_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = node as *mut AstStatAssign;
        unsafe {
            for rhs in (*node).values.iter() {
                self.mark_escaped(*rhs);
            }
            for lhs in (*node).vars.iter() {
                self.mark_escaped_table_index(*lhs, true);
            }
        }
        true
    }

    fn visit_stat_compound_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = node as *mut AstStatCompoundAssign;
        unsafe {
            self.mark_escaped_table_index((*node).var, true);
        }
        true
    }

    fn visit_stat_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = node as *mut AstStatFunction;
        unsafe {
            self.mark_escaped_table_index((*node).name, true);
        }
        true
    }

    fn visit_stat_for_in(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = node as *mut AstStatForIn;
        unsafe {
            for expr in (*node).values.iter() {
                self.mark_escaped(*expr);
            }
        }
        true
    }

    fn visit_stat_return(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = node as *mut AstStatReturn;
        unsafe {
            for expr in (*node).list.iter() {
                self.mark_escaped(*expr);
            }
        }
        true
    }
}
