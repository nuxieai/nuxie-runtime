use crate::enums::table_constant_kind::TableConstantKind;
use crate::records::variable::Variable;
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
use luaur_ast::records::ast_stat_assign::AstStatAssign;
use luaur_ast::records::ast_stat_compound_assign::AstStatCompoundAssign;
use luaur_ast::records::ast_stat_for_in::AstStatForIn;
use luaur_ast::records::ast_stat_function::AstStatFunction;
use luaur_ast::records::ast_stat_local::AstStatLocal;
use luaur_ast::records::ast_stat_return::AstStatReturn;
use luaur_ast::records::ast_visitor::AstVisitor;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::FFlag;

#[derive(Debug)]
pub struct TableMutationTrackerDeprecated<'a> {
    pub(crate) constant_tables:
        &'a mut DenseHashMap<*mut luaur_ast::records::ast_local::AstLocal, TableConstantKind>,
    pub(crate) variables: &'a DenseHashMap<*mut luaur_ast::records::ast_local::AstLocal, Variable>,
}

impl<'a> TableMutationTrackerDeprecated<'a> {
    pub fn table_mutation_tracker_deprecated(
        constant_tables: &'a mut DenseHashMap<
            *mut luaur_ast::records::ast_local::AstLocal,
            TableConstantKind,
        >,
        variables: &'a DenseHashMap<*mut luaur_ast::records::ast_local::AstLocal, Variable>,
    ) -> Self {
        LUAU_ASSERT!(FFlag::LuauCompilePropagateTableProps2.get());
        Self {
            constant_tables,
            variables,
        }
    }

    pub fn is_non_table_constant(&self, node: *mut AstExpr) -> bool {
        unsafe {
            if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprGroup>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                return self.is_non_table_constant(expr.expr as *mut AstExpr);
            }

            if luaur_ast::rtti::ast_node_is::<AstExprConstantNil>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return true;
            } else if luaur_ast::rtti::ast_node_is::<AstExprConstantBool>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return true;
            } else if luaur_ast::rtti::ast_node_is::<AstExprConstantNumber>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return true;
            } else if luaur_ast::rtti::ast_node_is::<AstExprConstantInteger>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return true;
            } else if luaur_ast::rtti::ast_node_is::<AstExprConstantString>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return true;
            } else if luaur_ast::rtti::ast_node_is::<AstExprLocal>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                let expr = &*(node as *mut AstExprLocal);
                if let Some(kind) = self.constant_tables.find(&expr.local) {
                    return *kind == TableConstantKind::ConstantOther;
                }
                return false;
            } else if luaur_ast::rtti::ast_node_is::<AstExprGlobal>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return false;
            } else if luaur_ast::rtti::ast_node_is::<AstExprVarargs>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return false;
            } else if luaur_ast::rtti::ast_node_is::<AstExprCall>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return false;
            } else if luaur_ast::rtti::ast_node_is::<AstExprIndexName>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                let expr = &*(node as *mut AstExprIndexName);
                let local = luaur_ast::rtti::ast_node_as::<AstExprLocal>(
                    expr.expr as *mut luaur_ast::records::ast_node::AstNode,
                );
                if local.is_null() {
                    return false;
                }
                let local_ref = &*local;
                if let Some(kind) = self.constant_tables.find(&local_ref.local) {
                    return *kind == TableConstantKind::ConstantTable;
                }
                return false;
            } else if luaur_ast::rtti::ast_node_is::<AstExprIndexExpr>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                let expr = &*(node as *mut AstExprIndexExpr);
                let local = luaur_ast::rtti::ast_node_as::<AstExprLocal>(
                    expr.expr as *mut luaur_ast::records::ast_node::AstNode,
                );
                if local.is_null() {
                    return false;
                }
                let local_ref = &*local;
                if let Some(kind) = self.constant_tables.find(&local_ref.local) {
                    return *kind == TableConstantKind::ConstantTable
                        && self.is_non_table_constant(expr.index as *mut AstExpr);
                }
                return false;
            } else if luaur_ast::rtti::ast_node_is::<AstExprFunction>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return false;
            } else if luaur_ast::rtti::ast_node_is::<AstExprTable>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return false;
            } else if luaur_ast::rtti::ast_node_is::<AstExprUnary>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                let expr = &*(node as *mut AstExprUnary);
                return self.is_non_table_constant(expr.expr as *mut AstExpr);
            } else if luaur_ast::rtti::ast_node_is::<AstExprBinary>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                let expr = &*(node as *mut AstExprBinary);
                return self.is_non_table_constant(expr.left as *mut AstExpr)
                    && self.is_non_table_constant(expr.right as *mut AstExpr);
            } else if luaur_ast::rtti::ast_node_is::<AstExprTypeAssertion>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                let expr = &*(node as *mut AstExprTypeAssertion);
                return self.is_non_table_constant(expr.expr as *mut AstExpr);
            } else if luaur_ast::rtti::ast_node_is::<AstExprIfElse>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                let expr = &*(node as *mut AstExprIfElse);
                return self.is_non_table_constant(expr.condition as *mut AstExpr)
                    && self.is_non_table_constant(expr.true_expr as *mut AstExpr)
                    && self.is_non_table_constant(expr.false_expr as *mut AstExpr);
            } else if luaur_ast::rtti::ast_node_is::<AstExprInterpString>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                let expr = &*(node as *mut AstExprInterpString);
                for i in 0..expr.expressions.size {
                    let expression = *expr.expressions.data.add(i);
                    if !self.is_non_table_constant(expression as *mut AstExpr) {
                        return false;
                    }
                }
                return true;
            } else if luaur_ast::rtti::ast_node_is::<AstExprInstantiate>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                let expr = &*(node as *mut AstExprInstantiate);
                return self.is_non_table_constant(expr.expr as *mut AstExpr);
            }

            LUAU_ASSERT!(false);
        }
        false
    }

    pub fn is_constant_table_literal(&self, node: *mut AstExpr) -> bool {
        unsafe {
            if let Some(table) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprTable>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                for i in 0..table.items.size {
                    let item = &*table.items.data.add(i);
                    if !item.key.is_null() {
                        if !self.is_non_table_constant(item.key as *mut AstExpr) {
                            return false;
                        }
                    }
                    if !self.is_non_table_constant(item.value as *mut AstExpr) {
                        return false;
                    }
                }
                return true;
            }

            if let Some(group) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprGroup>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                return self.is_constant_table_literal(group.expr as *mut AstExpr);
            }

            if let Some(assert) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprTypeAssertion>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                return self.is_constant_table_literal(assert.expr as *mut AstExpr);
            }

            if let Some(instantiate) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprInstantiate>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                return self.is_constant_table_literal(instantiate.expr as *mut AstExpr);
            }

            false
        }
    }

    pub fn could_be_table_reference(&self, node: *mut AstExpr) -> bool {
        unsafe {
            if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprGroup>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                return self.could_be_table_reference(expr.expr as *mut AstExpr);
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprTypeAssertion>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                return self.could_be_table_reference(expr.expr as *mut AstExpr);
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprInstantiate>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                return self.could_be_table_reference(expr.expr as *mut AstExpr);
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprIfElse>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                return self.could_be_table_reference(expr.true_expr as *mut AstExpr)
                    || self.could_be_table_reference(expr.false_expr as *mut AstExpr);
            } else if let Some(bin_expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprBinary>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                if bin_expr.op == luaur_ast::records::ast_expr_binary::AstExprBinaryOp::And
                    || bin_expr.op == luaur_ast::records::ast_expr_binary::AstExprBinaryOp::Or
                {
                    return self.could_be_table_reference(bin_expr.left as *mut AstExpr)
                        || self.could_be_table_reference(bin_expr.right as *mut AstExpr);
                }
            }

            if luaur_ast::rtti::ast_node_is::<AstExprLocal>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return true;
            }

            false
        }
    }

    pub fn observe_mutations(&mut self, node: *mut AstExpr, could_mutate_table: bool) {
        unsafe {
            if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprGroup>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                self.observe_mutations(expr.expr as *mut AstExpr, could_mutate_table);
            } else if luaur_ast::rtti::ast_node_is::<AstExprConstantNil>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return;
            } else if luaur_ast::rtti::ast_node_is::<AstExprConstantBool>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return;
            } else if luaur_ast::rtti::ast_node_is::<AstExprConstantNumber>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return;
            } else if luaur_ast::rtti::ast_node_is::<AstExprConstantInteger>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return;
            } else if luaur_ast::rtti::ast_node_is::<AstExprConstantString>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return;
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprLocal>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                let local = expr.local;
                if could_mutate_table && self.constant_tables.contains_key(&local) {
                    *self.constant_tables.get_or_insert(local) = TableConstantKind::NotConstant;
                }
            } else if luaur_ast::rtti::ast_node_is::<AstExprGlobal>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return;
            } else if luaur_ast::rtti::ast_node_is::<AstExprVarargs>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            ) {
                return;
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprCall>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                self.observe_mutations(expr.func as *mut AstExpr, true);
                for i in 0..expr.args.size {
                    let arg = *expr.args.data.add(i);
                    let could_mutate = self.could_be_table_reference(arg as *mut AstExpr);
                    self.observe_mutations(arg as *mut AstExpr, could_mutate);
                }
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprIndexName>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                self.observe_mutations(expr.expr as *mut AstExpr, could_mutate_table);
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprIndexExpr>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                self.observe_mutations(expr.index as *mut AstExpr, false);
                self.observe_mutations(expr.expr as *mut AstExpr, could_mutate_table);
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprFunction>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                luaur_ast::visit::ast_stat_visit(
                    expr.body as *mut luaur_ast::records::ast_stat::AstStat,
                    self,
                );
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprTable>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                for i in 0..expr.items.size {
                    let item = &*expr.items.data.add(i);
                    if !item.key.is_null() {
                        self.observe_mutations(item.key as *mut AstExpr, false);
                    }
                    self.observe_mutations(
                        item.value as *mut AstExpr,
                        self.could_be_table_reference(item.value as *mut AstExpr),
                    );
                }
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprUnary>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                self.observe_mutations(expr.expr as *mut AstExpr, false);
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprBinary>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                let short_circuiting = expr.op
                    == luaur_ast::records::ast_expr_binary::AstExprBinaryOp::And
                    || expr.op == luaur_ast::records::ast_expr_binary::AstExprBinaryOp::Or;
                self.observe_mutations(expr.left as *mut AstExpr, short_circuiting);
                self.observe_mutations(expr.right as *mut AstExpr, short_circuiting);
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprTypeAssertion>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                self.observe_mutations(expr.expr as *mut AstExpr, could_mutate_table);
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprIfElse>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                self.observe_mutations(expr.condition as *mut AstExpr, false);
                self.observe_mutations(expr.true_expr as *mut AstExpr, could_mutate_table);
                self.observe_mutations(expr.false_expr as *mut AstExpr, could_mutate_table);
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprInterpString>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                for i in 0..expr.expressions.size {
                    let expression = *expr.expressions.data.add(i);
                    self.observe_mutations(expression as *mut AstExpr, false);
                }
            } else if let Some(expr) = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprInstantiate>(
                    node as *mut luaur_ast::records::ast_node::AstNode,
                )
                .as_mut()
            } {
                self.observe_mutations(expr.expr as *mut AstExpr, could_mutate_table);
            } else {
                LUAU_ASSERT!(false);
            }
        }
    }
}

impl<'a> AstVisitor for TableMutationTrackerDeprecated<'a> {
    fn visit_expr(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = node as *mut AstExpr;
        self.observe_mutations(node, false);
        false
    }

    fn visit_stat_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        unsafe {
            let node = &*(node as *mut AstStatLocal);

            for i in 0..node.vars.size.min(node.values.size) {
                let local_ptr = *node.vars.data.add(i);
                let rhs = *node.values.data.add(i);

                let v = self.variables.find(&local_ptr);
                LUAU_ASSERT!(v.is_some());
                let v = v.unwrap();

                if !v.written {
                    if self.is_constant_table_literal(rhs as *mut AstExpr) {
                        *self.constant_tables.get_or_insert(local_ptr) =
                            TableConstantKind::ConstantTable;
                    } else if self.is_non_table_constant(rhs as *mut AstExpr) {
                        *self.constant_tables.get_or_insert(local_ptr) =
                            TableConstantKind::ConstantOther;
                    }
                }

                if !self.constant_tables.contains_key(&local_ptr) {
                    self.observe_mutations(
                        rhs as *mut AstExpr,
                        self.could_be_table_reference(rhs as *mut AstExpr),
                    );
                }
            }

            if node.vars.size < node.values.size {
                for i in node.vars.size..node.values.size {
                    let rhs = *node.values.data.add(i);
                    self.observe_mutations(rhs as *mut AstExpr, false);
                }
            }

            false
        }
    }

    fn visit_stat_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        unsafe {
            let node = &*(node as *mut AstStatAssign);

            for i in 0..node.vars.size.min(node.values.size) {
                let rhs = *node.values.data.add(i);
                self.observe_mutations(
                    rhs as *mut AstExpr,
                    self.could_be_table_reference(rhs as *mut AstExpr),
                );
            }

            if node.values.size > node.vars.size {
                for i in node.vars.size..node.values.size {
                    let rhs = *node.values.data.add(i);
                    self.observe_mutations(rhs as *mut AstExpr, false);
                }
            }

            for i in 0..node.vars.size {
                let lhs = *node.vars.data.add(i);
                self.observe_mutations(lhs as *mut AstExpr, true);
            }

            false
        }
    }

    fn visit_stat_compound_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        unsafe {
            let node = &*(node as *mut AstStatCompoundAssign);
            let rhs = node.value as *mut AstExpr;
            self.observe_mutations(rhs, self.could_be_table_reference(rhs));
            self.observe_mutations(node.var as *mut AstExpr, true);
            false
        }
    }

    fn visit_stat_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        unsafe {
            let node = &*(node as *mut AstStatFunction);
            self.observe_mutations(node.func as *mut AstExpr, false);
            self.observe_mutations(node.name as *mut AstExpr, true);
            false
        }
    }

    fn visit_stat_return(&mut self, node: *mut core::ffi::c_void) -> bool {
        unsafe {
            let node = &*(node as *mut AstStatReturn);
            for i in 0..node.list.size {
                let expr = *node.list.data.add(i);
                self.observe_mutations(
                    expr as *mut AstExpr,
                    self.could_be_table_reference(expr as *mut AstExpr),
                );
            }
            false
        }
    }

    fn visit_stat_for_in(&mut self, node: *mut core::ffi::c_void) -> bool {
        unsafe {
            let node = &*(node as *mut AstStatForIn);

            for i in 0..node.values.size {
                let expr = *node.values.data.add(i);
                self.observe_mutations(expr as *mut AstExpr, true);
            }

            luaur_ast::visit::ast_stat_visit(
                node.body as *mut luaur_ast::records::ast_stat::AstStat,
                self,
            );
            false
        }
    }
}
