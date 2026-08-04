//! Source: `Compiler/src/ConstantFolding.cpp:944-1396`

use crate::enums::table_constant_kind::TableConstantKind;
use crate::records::constant::Constant;
use crate::records::variable::Variable;
use crate::type_aliases::expr_constant_change_log::ExprConstantChangeLog;
use crate::type_aliases::library_member_constant_callback::LibraryMemberConstantCallback;
use crate::type_aliases::local_constant_change_log::LocalConstantChangeLog;
use alloc::vec::Vec;
use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_table::Item;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::ast_visitor::AstVisitor;
use luaur_common::records::dense_hash_map::DenseHashMap;

#[derive(Debug)]
pub struct ConstantVisitor<'a> {
    pub(crate) constants: &'a mut DenseHashMap<*mut AstExpr, Constant>,
    pub(crate) variables: &'a mut DenseHashMap<*mut AstLocal, Variable>,
    pub(crate) locals: &'a mut DenseHashMap<*mut AstLocal, Constant>,
    pub(crate) builtins: *const DenseHashMap<*mut AstExprCall, i32>,
    pub(crate) fold_library_k: bool,
    pub(crate) library_member_constant_cb: LibraryMemberConstantCallback,
    pub(crate) string_table: &'a mut AstNameTable,
    pub(crate) constant_tables: Vec<DenseHashMap<AstName, Constant>>,
    pub(crate) was_empty: bool,
    pub(crate) builtin_args: Vec<Constant>,
    pub(crate) constant_table_locals: &'a DenseHashMap<*mut AstLocal, TableConstantKind>,
    pub(crate) table_locals: DenseHashMap<*mut AstLocal, Constant>,
    pub(crate) expr_change_log: *mut ExprConstantChangeLog,
    pub(crate) local_change_log: *mut LocalConstantChangeLog,
}

impl<'a> ConstantVisitor<'a> {
    pub fn constant_visitor(
        constants: &'a mut DenseHashMap<*mut AstExpr, Constant>,
        variables: &'a mut DenseHashMap<*mut AstLocal, Variable>,
        locals: &'a mut DenseHashMap<*mut AstLocal, Constant>,
        builtins: *const DenseHashMap<*mut AstExprCall, i32>,
        fold_library_k: bool,
        library_member_constant_cb: LibraryMemberConstantCallback,
        string_table: &'a mut AstNameTable,
        constant_table_locals: &'a DenseHashMap<*mut AstLocal, TableConstantKind>,
        expr_change_log: *mut ExprConstantChangeLog,
        local_change_log: *mut LocalConstantChangeLog,
    ) -> Self {
        let mut constant_tables = Vec::new();
        constant_tables.reserve(16);

        let table_locals = DenseHashMap::new(core::ptr::null_mut());

        let was_empty = constants.empty() && locals.empty();

        Self {
            constants,
            variables,
            locals,
            builtins,
            fold_library_k,
            library_member_constant_cb,
            string_table,
            constant_tables,
            was_empty,
            builtin_args: Vec::new(),
            constant_table_locals,
            table_locals,
            expr_change_log,
            local_change_log,
        }
    }

    fn analyze(&mut self, node: *mut AstExpr) -> Constant {
        let mut result = Constant::default();
        result.r#type = crate::enums::type_constant_folding::Type::Type_Unknown;

        let node_ref = unsafe { &*node };

        if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_group::AstExprGroup>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            result = self.analyze(expr.expr);
        } else if luaur_ast::rtti::ast_node_is::<
            luaur_ast::records::ast_expr_constant_nil::AstExprConstantNil,
        >(node as *mut luaur_ast::records::ast_node::AstNode)
        {
            result.r#type = crate::enums::type_constant_folding::Type::Type_Nil;
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<
                luaur_ast::records::ast_expr_constant_bool::AstExprConstantBool,
            >(node as *mut luaur_ast::records::ast_node::AstNode)
            .as_mut()
        } {
            result.r#type = crate::enums::type_constant_folding::Type::Type_Boolean;
            result.data.value_boolean = expr.value;
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<
                luaur_ast::records::ast_expr_constant_number::AstExprConstantNumber,
            >(node as *mut luaur_ast::records::ast_node::AstNode)
            .as_mut()
        } {
            result.r#type = crate::enums::type_constant_folding::Type::Type_Number;
            result.data.value_number = expr.value;
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<
                luaur_ast::records::ast_expr_constant_integer::AstExprConstantInteger,
            >(node as *mut luaur_ast::records::ast_node::AstNode)
            .as_mut()
        } {
            result.r#type = crate::enums::type_constant_folding::Type::Type_Integer;
            result.data.value_integer64 = expr.value;
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<
                luaur_ast::records::ast_expr_constant_string::AstExprConstantString,
            >(node as *mut luaur_ast::records::ast_node::AstNode)
            .as_mut()
        } {
            result.r#type = crate::enums::type_constant_folding::Type::Type_String;
            result.data.value_string = expr.value.data as *const core::ffi::c_char;
            result.string_length = expr.value.size as u32;
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_local::AstExprLocal>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            if let Some(l) = self.locals.find(&expr.local) {
                result = *l;
            } else if luaur_common::FFlag::LuauCompileFoldOptimize.get() {
                if let Some(l) = self.table_locals.find(&expr.local) {
                    result = *l;
                }
            }
        } else if luaur_ast::rtti::ast_node_is::<luaur_ast::records::ast_expr_global::AstExprGlobal>(
            node as *mut luaur_ast::records::ast_node::AstNode,
        ) {
            // nope
        } else if luaur_ast::rtti::ast_node_is::<luaur_ast::records::ast_expr_varargs::AstExprVarargs>(
            node as *mut luaur_ast::records::ast_node::AstNode,
        ) {
            // nope
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_call::AstExprCall>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            self.analyze(expr.func);

            let bfid = if !self.builtins.is_null() {
                unsafe { &*self.builtins }
                    .find(&(expr as *mut luaur_ast::records::ast_expr_call::AstExprCall))
            } else {
                None
            };

            if let Some(bfid_ptr) = bfid {
                if *bfid_ptr != 0 {
                    let offset = self.builtin_args.len();
                    let mut can_fold = true;

                    self.builtin_args.reserve(offset + expr.args.size as usize);

                    for i in 0..expr.args.size as usize {
                        let ac = self.analyze(unsafe { *expr.args.data.add(i) });

                        if luaur_common::FFlag::LuauCompilePropagateTableProps2.get() {
                            if ac.r#type == crate::enums::type_constant_folding::Type::Type_Unknown
                                || ac.r#type
                                    == crate::enums::type_constant_folding::Type::Type_Table
                            {
                                can_fold = false;
                            } else {
                                self.builtin_args.push(ac);
                            }
                        } else if ac.r#type
                            == crate::enums::type_constant_folding::Type::Type_Unknown
                        {
                            can_fold = false;
                        } else {
                            self.builtin_args.push(ac);
                        }
                    }

                    if can_fold {
                        luaur_common::LUAU_ASSERT!(
                            self.builtin_args.len() == offset + expr.args.size as usize
                        );
                        result = crate::functions::fold_builtin::fold_builtin(
                            self.string_table,
                            *bfid_ptr,
                            unsafe { self.builtin_args.as_ptr().add(offset) },
                            expr.args.size as usize,
                        );
                    }

                    self.builtin_args.resize(offset, Constant::default());
                } else {
                    for i in 0..expr.args.size as usize {
                        self.analyze(unsafe { *expr.args.data.add(i) });
                    }
                }
            } else {
                for i in 0..expr.args.size as usize {
                    self.analyze(unsafe { *expr.args.data.add(i) });
                }
            }
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_index_name::AstExprIndexName>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let value = self.analyze(expr.expr);
            if luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
                && value.r#type == crate::enums::type_constant_folding::Type::Type_Table
            {
                let table_index = unsafe { value.data.value_table };
                luaur_common::LUAU_ASSERT!(table_index < self.constant_tables.len());
                if table_index < self.constant_tables.len() {
                    let props = &self.constant_tables[table_index];
                    if let Some(prop) = props.find(&expr.index) {
                        result = *prop;
                    }
                }
            } else if value.r#type == crate::enums::type_constant_folding::Type::Type_Vector {
                let index_str =
                    unsafe { core::ffi::CStr::from_ptr(expr.index.value).to_string_lossy() };
                if index_str == "x" || index_str == "X" {
                    result.r#type = crate::enums::type_constant_folding::Type::Type_Number;
                    result.data.value_number = unsafe { value.data.value_vector[0] as f64 };
                } else if index_str == "y" || index_str == "Y" {
                    result.r#type = crate::enums::type_constant_folding::Type::Type_Number;
                    result.data.value_number = unsafe { value.data.value_vector[1] as f64 };
                } else if index_str == "z" || index_str == "Z" {
                    result.r#type = crate::enums::type_constant_folding::Type::Type_Number;
                    result.data.value_number = unsafe { value.data.value_vector[2] as f64 };
                }

                // Do not handle 'w' component because it isn't known if the runtime will be configured in 3-wide or 4-wide mode
                // In 3-wide, access to 'w' will call unspecified metamethod or fail
            } else if self.fold_library_k {
                if let Some(eg) = unsafe {
                    luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_global::AstExprGlobal>(expr.expr as *mut luaur_ast::records::ast_node::AstNode).as_mut()
                } {
                    let eg_name =
                        unsafe { core::ffi::CStr::from_ptr(eg.name.value).to_string_lossy() };
                    let index_str =
                        unsafe { core::ffi::CStr::from_ptr(expr.index.value).to_string_lossy() };
                    if eg_name == "math" {
                        result = crate::functions::fold_builtin_math::fold_builtin_math(expr.index);
                    }

                    if self.library_member_constant_cb.is_some()
                        && result.r#type == crate::enums::type_constant_folding::Type::Type_Unknown
                    {
                        let cb = self.library_member_constant_cb.unwrap();
                        // C++ passes reinterpret_cast<CompileConstant*>(&result): the
                        // pointer VALUE handed to the callback must be &result itself.
                        let constant_ptr = &mut result as *mut Constant
                            as *mut crate::type_aliases::compile_constant::CompileConstant;
                        unsafe { cb(eg.name.value, expr.index.value, constant_ptr) };
                    }
                }
            }
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_index_expr::AstExprIndexExpr>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let index_val = self.analyze(expr.index);
            let table_val = self.analyze(expr.expr);

            if luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
                && table_val.r#type == crate::enums::type_constant_folding::Type::Type_Table
                && index_val.r#type == crate::enums::type_constant_folding::Type::Type_String
            {
                let table_index = unsafe { table_val.data.value_table };
                luaur_common::LUAU_ASSERT!(table_index < self.constant_tables.len());
                if table_index < self.constant_tables.len() && index_val.string_length != 0 {
                    let props = &self.constant_tables[table_index];
                    let index_name = self.string_table.get_or_add(
                        unsafe { index_val.data.value_string } as *const core::ffi::c_char,
                        index_val.string_length as usize,
                    );
                    if let Some(prop) = props.find(&index_name) {
                        result = *prop;
                    }
                }
            }
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_function::AstExprFunction>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            unsafe {
                luaur_ast::visit::ast_stat_visit(
                    expr.body as *mut luaur_ast::records::ast_stat::AstStat,
                    self as &mut dyn AstVisitor,
                );
            }
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_table::AstExprTable>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            if luaur_common::FFlag::LuauCompilePropagateTableProps2.get() {
                let mut props = DenseHashMap::new(AstName::new());
                for i in 0..expr.items.size as usize {
                    let item = unsafe { &*expr.items.data.add(i) };

                    let value_val = self.analyze(item.value);

                    if !item.key.is_null() {
                        let key_val = self.analyze(item.key);

                        if key_val.r#type == crate::enums::type_constant_folding::Type::Type_String
                            && value_val.r#type
                                != crate::enums::type_constant_folding::Type::Type_Unknown
                            && value_val.r#type
                                != crate::enums::type_constant_folding::Type::Type_Table
                            && key_val.string_length != 0
                        {
                            let const_key = self.string_table.get_or_add(
                                unsafe { key_val.data.value_string } as *const core::ffi::c_char,
                                key_val.string_length as usize,
                            );
                            props.try_insert(const_key, value_val);
                        }
                    }
                }

                if props.size() == expr.items.size as usize {
                    result.r#type = crate::enums::type_constant_folding::Type::Type_Table;
                    result.data.value_table = self.constant_tables.len();
                    self.constant_tables.push(props);
                }
            } else {
                for i in 0..expr.items.size as usize {
                    let item = unsafe { &*expr.items.data.add(i) };

                    if !item.key.is_null() {
                        self.analyze(item.key);
                    }

                    self.analyze(item.value);
                }
            }
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_unary::AstExprUnary>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let arg = self.analyze(expr.expr);

            if arg.r#type != crate::enums::type_constant_folding::Type::Type_Unknown {
                crate::functions::fold_unary::fold_unary(&mut result, expr.op, &arg);
            }
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_binary::AstExprBinary>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let la = self.analyze(expr.left);
            let ra = self.analyze(expr.right);

            if la.r#type != crate::enums::type_constant_folding::Type::Type_Unknown {
                crate::functions::fold_binary::fold_binary(
                    &mut result,
                    expr.op,
                    &la,
                    &ra,
                    self.string_table,
                );
            }
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<
                luaur_ast::records::ast_expr_type_assertion::AstExprTypeAssertion,
            >(node as *mut luaur_ast::records::ast_node::AstNode)
            .as_mut()
        } {
            let arg = self.analyze(expr.expr);
            result = arg;
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<luaur_ast::records::ast_expr_if_else::AstExprIfElse>(
                node as *mut luaur_ast::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            let cond = self.analyze(expr.condition);
            let true_expr = self.analyze(expr.true_expr);
            let false_expr = self.analyze(expr.false_expr);

            if cond.r#type != crate::enums::type_constant_folding::Type::Type_Unknown {
                result = if cond.is_truthful() {
                    true_expr
                } else {
                    false_expr
                };
            }
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<
                luaur_ast::records::ast_expr_interp_string::AstExprInterpString,
            >(node as *mut luaur_ast::records::ast_node::AstNode)
            .as_mut()
        } {
            // C++ analyzes EVERY sub-expression (no early break) so that constants
            // are recorded for all of them — including local references nested in
            // later, non-constant interpolations. A `break` here left those refs
            // unfolded; a constant local whose declaration is then elided
            // (areLocalsRedundant) would reach compile_expr with no register and
            // trip the upvalue assert.
            let mut only_constant_sub_expr = true;
            for i in 0..expr.expressions.size as usize {
                if self
                    .analyze(unsafe { *expr.expressions.data.add(i) })
                    .r#type
                    != crate::enums::type_constant_folding::Type::Type_String
                {
                    only_constant_sub_expr = false;
                }
            }

            if only_constant_sub_expr {
                crate::functions::fold_interp_string::fold_interp_string(
                    &mut result,
                    expr,
                    self.constants,
                    self.string_table,
                );
            }
        } else if let Some(expr) = unsafe {
            luaur_ast::rtti::ast_node_as::<
                luaur_ast::records::ast_expr_instantiate::AstExprInstantiate,
            >(node as *mut luaur_ast::records::ast_node::AstNode)
            .as_mut()
        } {
            result = self.analyze(expr.expr);
        } else {
            luaur_common::LUAU_ASSERT!(false, "Unknown expression type");
        }

        self.record_expr_constant(node, result);

        result
    }

    fn record_expr_constant(&mut self, key: *mut AstExpr, value: Constant) {
        if luaur_common::FFlag::LuauCompileFoldOptimize.get()
            && luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
        {
            if value.r#type == crate::enums::type_constant_folding::Type::Type_Table {
                // Table constants are recorded in a separate map
            } else if value.r#type != crate::enums::type_constant_folding::Type::Type_Unknown {
                self.log_expr_change(key, None);
                *self.constants.get_or_insert(key) = value;
            } else if self.was_empty {
                // No need to clear out entries if we started with empty maps
            } else if let Some(old) = self.constants.find(&key).copied() {
                self.log_expr_change(key, Some(old));
                // C++ `old->type = Type_Unknown`: clear the STALE entry. try_insert is a no-op
                // when the key exists, so the stale constant survived across inline re-folds.
                *self.constants.get_or_insert(key) = Constant::default();
            }
        } else {
            if value.r#type != crate::enums::type_constant_folding::Type::Type_Unknown {
                *self.constants.get_or_insert(key) = value;
            } else if self.was_empty && !luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
            {
                // nothing
            } else if self.constants.find(&key).is_some() {
                // C++ `old->type = Type_Unknown`: clear the STALE entry. try_insert is a no-op
                // when the key exists, so the stale constant survived across inline re-folds.
                *self.constants.get_or_insert(key) = Constant::default();
            }
        }
    }

    fn record_local_constant(&mut self, key: *mut AstLocal, value: Constant) {
        if luaur_common::FFlag::LuauCompileFoldOptimize.get()
            && luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
        {
            if value.r#type == crate::enums::type_constant_folding::Type::Type_Table {
                // Table constants are recorded in a separate map
            } else if value.r#type != crate::enums::type_constant_folding::Type::Type_Unknown {
                self.log_local_change(key, None);
                *self.locals.get_or_insert(key) = value;
            } else if self.was_empty {
                // No need to clear out entries if we started with empty maps
            } else if let Some(old) = self.locals.find(&key).copied() {
                self.log_local_change(key, Some(old));
                *self.locals.get_or_insert(key) = Constant::default();
            }
        } else {
            if value.r#type != crate::enums::type_constant_folding::Type::Type_Unknown {
                *self.locals.get_or_insert(key) = value;
            } else if self.was_empty && !luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
            {
                // nothing
            } else if self.locals.find(&key).is_some() {
                *self.locals.get_or_insert(key) = Constant::default();
            }
        }
    }

    fn log_expr_change(&mut self, key: *mut AstExpr, existing: Option<Constant>) {
        if self.expr_change_log.is_null() {
            return;
        }

        let old = existing
            .or_else(|| self.constants.find(&key).copied())
            .unwrap_or_default();
        let was_absent = existing.is_none() && self.constants.find(&key).is_none();

        let log = unsafe { &mut *self.expr_change_log };
        log.push(crate::records::expr_constant_change::ExprConstantChange {
            key,
            old_value: old,
            was_absent,
        });
    }

    fn log_local_change(&mut self, key: *mut AstLocal, existing: Option<Constant>) {
        if self.local_change_log.is_null() {
            return;
        }

        let old = existing
            .or_else(|| self.locals.find(&key).copied())
            .unwrap_or_default();
        let was_absent = existing.is_none() && self.locals.find(&key).is_none();

        let log = unsafe { &mut *self.local_change_log };
        log.push(crate::records::local_constant_change::LocalConstantChange {
            key,
            old_value: old,
            was_absent,
        });
    }

    fn record_value(&mut self, local: *mut AstLocal, value: Constant) {
        let v = self.variables.find_mut(&local).unwrap();

        if !v.written {
            if luaur_common::FFlag::LuauCompileFoldOptimize.get()
                && luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
            {
                if value.r#type == crate::enums::type_constant_folding::Type::Type_Table {
                    v.constant = false;
                    self.table_locals.try_insert(local, value);
                } else {
                    v.constant =
                        value.r#type != crate::enums::type_constant_folding::Type::Type_Unknown;
                    self.record_local_constant(local, value);
                }
            } else {
                v.constant = if luaur_common::FFlag::LuauCompilePropagateTableProps2.get() {
                    value.r#type != crate::enums::type_constant_folding::Type::Type_Unknown
                        && value.r#type != crate::enums::type_constant_folding::Type::Type_Table
                } else {
                    value.r#type != crate::enums::type_constant_folding::Type::Type_Unknown
                };
                self.record_local_constant(local, value);
            }
        }
    }

    fn visit_stat_local(
        &mut self,
        node: *mut luaur_ast::records::ast_stat_local::AstStatLocal,
    ) -> bool {
        let node_ref = unsafe { &*node };

        for i in 0..node_ref.vars.size.min(node_ref.values.size) {
            let rhs = unsafe { *node_ref.values.data.add(i) };
            let arg = self.analyze(rhs);

            if luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
                && arg.r#type == crate::enums::type_constant_folding::Type::Type_Table
            {
                let local = unsafe { *node_ref.vars.data.add(i) };

                let kind = self.constant_table_locals.find(&local);
                if let Some(k) = kind {
                    if *k == crate::enums::table_constant_kind::TableConstantKind::ConstantTable {
                        self.record_value(local, arg);
                    } else {
                        self.record_value(local, Constant::default());
                    }
                } else {
                    self.record_value(local, Constant::default());
                }
            } else {
                self.record_value(unsafe { *node_ref.vars.data.add(i) }, arg);
            }
        }

        if node_ref.vars.size > node_ref.values.size {
            let last = if node_ref.values.size > 0 {
                Some(unsafe { *node_ref.values.data.add(node_ref.values.size as usize - 1) })
            } else {
                None
            };
            let mult_ret = last.map_or(false, |l| {
                luaur_ast::rtti::ast_node_is::<luaur_ast::records::ast_expr_call::AstExprCall>(
                    l as *mut luaur_ast::records::ast_node::AstNode,
                ) || luaur_ast::rtti::ast_node_is::<
                    luaur_ast::records::ast_expr_varargs::AstExprVarargs,
                >(l as *mut luaur_ast::records::ast_node::AstNode)
            });

            if !mult_ret {
                for i in node_ref.values.size..node_ref.vars.size {
                    let nil = Constant {
                        r#type: crate::enums::type_constant_folding::Type::Type_Nil,
                        string_length: 0,
                        data: Default::default(),
                    };
                    self.record_value(unsafe { *node_ref.vars.data.add(i as usize) }, nil);
                }
            }
        } else {
            for i in node_ref.vars.size..node_ref.values.size {
                self.analyze(unsafe { *node_ref.values.data.add(i as usize) });
            }
        }

        false
    }
}

impl<'a> AstVisitor for ConstantVisitor<'a> {
    fn visit_node(&mut self, node: *mut core::ffi::c_void) -> bool {
        true
    }

    fn visit_expr(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.analyze(node as *mut AstExpr);
        false
    }

    fn visit_stat_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat_local(node as *mut luaur_ast::records::ast_stat_local::AstStatLocal);
        false
    }
}
