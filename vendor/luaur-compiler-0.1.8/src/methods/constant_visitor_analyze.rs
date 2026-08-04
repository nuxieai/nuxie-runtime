use crate::enums::type_constant_folding::Type;
use crate::functions::fold_binary::fold_binary;
use crate::functions::fold_builtin::fold_builtin;
use crate::functions::fold_builtin_math::fold_builtin_math;
use crate::functions::fold_interp_string::fold_interp_string;
use crate::functions::fold_unary::fold_unary;
use crate::records::constant::Constant;
use crate::records::constant_visitor::ConstantVisitor;
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
use luaur_ast::records::ast_name::AstName;
use luaur_ast::rtti::{ast_node_as, ast_node_is};
use luaur_ast::visit::AstVisitable;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> ConstantVisitor<'a> {
    pub fn analyze(&mut self, node: *mut AstExpr) -> Constant {
        let mut result = Constant {
            r#type: Type::Type_Unknown,
            string_length: 0,
            data: unsafe { core::mem::zeroed() },
        };

        if node.is_null() {
            return result;
        }

        let node_ptr = node as *mut luaur_ast::records::ast_node::AstNode;

        if let Some(expr) = unsafe { ast_node_as::<AstExprGroup>(node_ptr).as_mut() } {
            result = self.analyze(expr.expr);
        } else if unsafe { ast_node_is::<AstExprConstantNil>(&*node_ptr) } {
            result.r#type = Type::Type_Nil;
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprConstantBool>(node_ptr).as_mut() } {
            result.r#type = Type::Type_Boolean;
            result.data.value_boolean = expr.value;
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprConstantNumber>(node_ptr).as_mut() } {
            result.r#type = Type::Type_Number;
            result.data.value_number = expr.value;
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprConstantInteger>(node_ptr).as_mut() } {
            result.r#type = Type::Type_Integer;
            result.data.value_integer64 = expr.value;
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprConstantString>(node_ptr).as_mut() } {
            result.r#type = Type::Type_String;
            result.data.value_string = expr.value.data;
            result.string_length = expr.value.size as core::ffi::c_uint;
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprLocal>(node_ptr).as_mut() } {
            if let Some(l) = self.locals.find(&expr.local) {
                result = *l;
            } else if luaur_common::FFlag::LuauCompileFoldOptimize.get() {
                if let Some(l) = self.table_locals.find(&expr.local) {
                    result = *l;
                }
            }
        } else if unsafe { ast_node_is::<AstExprGlobal>(&*node_ptr) }
            || unsafe { ast_node_is::<AstExprVarargs>(&*node_ptr) }
        {
            // nope
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprCall>(node_ptr).as_mut() } {
            self.analyze(expr.func);
            let bfid = if !self.builtins.is_null() {
                let key = &(expr as *mut AstExprCall);
                unsafe { (*self.builtins).find(key) }
            } else {
                None
            };
            if let Some(&id) = bfid {
                if id != 0 {
                    let offset = self.builtin_args.len();
                    let mut can_fold = true;
                    self.builtin_args.reserve(offset + expr.args.size);
                    for i in 0..expr.args.size {
                        let ac = self.analyze(unsafe { *expr.args.data.add(i) });
                        let unknown = ac.r#type == Type::Type_Unknown;
                        let table = ac.r#type == Type::Type_Table;
                        let propagate = luaur_common::FFlag::LuauCompilePropagateTableProps2.get();
                        if (propagate && (unknown || table)) || (!propagate && unknown) {
                            can_fold = false;
                        } else {
                            self.builtin_args.push(ac);
                        }
                    }
                    if can_fold {
                        LUAU_ASSERT!(self.builtin_args.len() == offset + expr.args.size);
                        result = fold_builtin(
                            self.string_table,
                            id,
                            unsafe { self.builtin_args.as_ptr().add(offset) },
                            expr.args.size,
                        );
                    }
                    self.builtin_args.truncate(offset);
                }
            } else {
                for i in 0..expr.args.size {
                    self.analyze(unsafe { *expr.args.data.add(i) });
                }
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprIndexName>(node_ptr).as_mut() } {
            let value = self.analyze(expr.expr);
            if luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
                && value.r#type == Type::Type_Table
            {
                let table_idx = unsafe { value.data.value_table };
                if table_idx < self.constant_tables.len() {
                    let props = &self.constant_tables[table_idx];
                    if let Some(prop) = props.find(&expr.index) {
                        result = *prop;
                    }
                }
            } else if value.r#type == Type::Type_Vector {
                if expr.index.operator_eq_c_char(c"x".as_ptr())
                    || expr.index.operator_eq_c_char(c"X".as_ptr())
                {
                    result.r#type = Type::Type_Number;
                    result.data.value_number = unsafe { value.data.value_vector[0] } as f64;
                } else if expr.index.operator_eq_c_char(c"y".as_ptr())
                    || expr.index.operator_eq_c_char(c"Y".as_ptr())
                {
                    result.r#type = Type::Type_Number;
                    result.data.value_number = unsafe { value.data.value_vector[1] } as f64;
                } else if expr.index.operator_eq_c_char(c"z".as_ptr())
                    || expr.index.operator_eq_c_char(c"Z".as_ptr())
                {
                    result.r#type = Type::Type_Number;
                    result.data.value_number = unsafe { value.data.value_vector[2] } as f64;
                }
            } else if self.fold_library_k {
                if let Some(eg) = unsafe {
                    ast_node_as::<AstExprGlobal>(expr.expr as *mut luaur_ast::records::ast_node::AstNode)
                        .as_mut()
                } {
                    if eg.name.operator_eq_c_char(c"math".as_ptr()) {
                        result = fold_builtin_math(expr.index);
                    }
                    if !self.library_member_constant_cb.is_none() && result.r#type == Type::Type_Unknown
                    {
                        unsafe {
                            (self.library_member_constant_cb.unwrap())(
                                eg.name.value,
                                expr.index.value,
                                &mut result as *mut Constant
                                    as *mut crate::type_aliases::compile_constant::CompileConstant,
                            );
                        }
                    }
                }
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprIndexExpr>(node_ptr).as_mut() } {
            let index_val = self.analyze(expr.index);
            let table_val = self.analyze(expr.expr);
            if luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
                && table_val.r#type == Type::Type_Table
                && index_val.r#type == Type::Type_String
            {
                let table_idx = unsafe { table_val.data.value_table };
                if table_idx < self.constant_tables.len() && index_val.string_length != 0 {
                    let props = &self.constant_tables[table_idx];
                    let index_name = self.string_table.get_or_add(
                        unsafe { index_val.data.value_string },
                        index_val.string_length as usize,
                    );
                    if let Some(prop) = props.find(&index_name) {
                        result = *prop;
                    }
                }
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprFunction>(node_ptr).as_mut() } {
            unsafe {
                (*expr.body).visit(self as &mut dyn luaur_ast::records::ast_visitor::AstVisitor);
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprTable>(node_ptr).as_mut() } {
            if luaur_common::FFlag::LuauCompilePropagateTableProps2.get() {
                let mut props =
                    luaur_common::records::dense_hash_map::DenseHashMap::new(AstName::new());
                for i in 0..expr.items.size {
                    let item = unsafe { &*expr.items.data.add(i) };
                    let value_val = self.analyze(item.value);
                    if !item.key.is_null() {
                        let key_val = self.analyze(item.key);
                        if key_val.r#type == Type::Type_String
                            && value_val.r#type != Type::Type_Unknown
                            && value_val.r#type != Type::Type_Table
                            && key_val.string_length != 0
                        {
                            let const_key = self.string_table.get_or_add(
                                unsafe { key_val.data.value_string },
                                key_val.string_length as usize,
                            );
                            props.try_insert(const_key, value_val);
                        }
                    }
                }
                if props.size() == expr.items.size {
                    result.r#type = Type::Type_Table;
                    result.data.value_table = self.constant_tables.len();
                    self.constant_tables.push(props);
                }
            } else {
                for i in 0..expr.items.size {
                    let item = unsafe { &*expr.items.data.add(i) };
                    if !item.key.is_null() {
                        self.analyze(item.key);
                    }
                    self.analyze(item.value);
                }
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprUnary>(node_ptr).as_mut() } {
            let arg = self.analyze(expr.expr);
            if arg.r#type != Type::Type_Unknown {
                fold_unary(&mut result, unsafe { core::ptr::read(expr) }, &arg);
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprBinary>(node_ptr).as_mut() } {
            let la = self.analyze(expr.left);
            let ra = self.analyze(expr.right);
            if la.r#type != Type::Type_Unknown {
                fold_binary(&mut result, expr.op, &la, &ra, self.string_table);
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprTypeAssertion>(node_ptr).as_mut() }
        {
            result = self.analyze(expr.expr);
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprIfElse>(node_ptr).as_mut() } {
            let cond = self.analyze(expr.condition);
            let true_expr = self.analyze(expr.true_expr);
            let false_expr = self.analyze(expr.false_expr);
            if cond.r#type != Type::Type_Unknown {
                result = if cond.is_truthful() {
                    true_expr
                } else {
                    false_expr
                };
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprInterpString>(node_ptr).as_mut() } {
            let mut only_constant = true;
            for i in 0..expr.expressions.size {
                if self.analyze(unsafe { *expr.expressions.data.add(i) }).r#type != Type::Type_String
                {
                    only_constant = false;
                }
            }
            if only_constant {
                fold_interp_string(&mut result, expr, self.constants, self.string_table);
            }
        } else if let Some(expr) = unsafe { ast_node_as::<AstExprInstantiate>(node_ptr).as_mut() } {
            result = self.analyze(expr.expr);
        } else {
            LUAU_ASSERT!(false);
        }

        let constants_ptr = self.constants as *mut _;
        self.record_constant(unsafe { &mut *constants_ptr }, node, &result);
        result
    }
}
