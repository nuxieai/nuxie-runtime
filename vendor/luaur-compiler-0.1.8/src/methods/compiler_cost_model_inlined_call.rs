use crate::enums::type_constant_folding::Type;
use crate::functions::fold_constants::fold_constants;
use crate::functions::model_cost_cost_model::model_cost_ast_node_ast_local_usize_dense_hash_map_ast_expr_call_i32_dense_hash_map_ast_expr_constant;
use crate::functions::undo_changes_constant_folding::undo_changes_dense_hash_map_ast_expr_constant_expr_constant_change_log;
use crate::functions::undo_changes_constant_folding_alt_b::undo_changes_dense_hash_map_ast_local_constant_local_constant_change_log;
use crate::records::compiler::Compiler;
use crate::records::constant::{Constant, ConstantData};
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_node::AstNode;

impl Compiler {
    pub fn cost_model_inlined_call(
        &mut self,
        expr: *mut AstExprCall,
        func: *mut AstExprFunction,
    ) -> u64 {
        unsafe {
            let func_ref = &*func;
            let expr_ref = &*expr;

            for i in 0..func_ref.args.size {
                let var = *func_ref.args.data.add(i);
                let arg = if i < expr_ref.args.size {
                    *expr_ref.args.data.add(i)
                } else {
                    core::ptr::null_mut()
                };

                if i + 1 == expr_ref.args.size
                    && func_ref.args.size > expr_ref.args.size
                    && self.is_expr_mult_ret(arg)
                {
                    break;
                }

                if self.variables.find(&var).map_or(false, |vv| vv.written) {
                    continue;
                }

                if arg.is_null() {
                    *self.locstants.get_or_insert(var) = Constant {
                        r#type: Type::Type_Nil,
                        string_length: 0,
                        data: ConstantData::default(),
                    };
                } else if let Some(cv) = self.constants.find(&arg) {
                    if cv.r#type != Type::Type_Unknown {
                        *self.locstants.get_or_insert(var) = *cv;
                    }
                }
            }

            let record_changes = luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
                && luaur_common::FFlag::LuauCompileFoldOptimize.get();

            if record_changes {
                self.expr_changes.clear();
                self.local_changes.clear();
            }

            fold_constants(
                &mut self.constants,
                &mut self.variables,
                &mut self.locstants,
                self.builtins_fold,
                self.builtins_fold_library_k,
                self.options.library_member_constant_cb,
                func_ref.body as *mut AstNode,
                &mut *self.names,
                &self.table_constants,
                if record_changes {
                    &mut self.expr_changes as *mut _
                } else {
                    core::ptr::null_mut()
                },
                if record_changes {
                    &mut self.local_changes as *mut _
                } else {
                    core::ptr::null_mut()
                },
            );

            let cost = model_cost_ast_node_ast_local_usize_dense_hash_map_ast_expr_call_i32_dense_hash_map_ast_expr_constant(
                func_ref.body as *mut AstNode,
                func_ref.args.data as *const _,
                func_ref.args.size,
                &self.builtins,
                &self.constants,
            );

            for i in 0..func_ref.args.size {
                let arg = *func_ref.args.data.add(i);
                if let Some(var) = self.locstants.find_mut(&arg) {
                    var.r#type = Type::Type_Unknown;
                }
            }

            if record_changes {
                undo_changes_dense_hash_map_ast_expr_constant_expr_constant_change_log(
                    &mut self.constants,
                    &self.expr_changes,
                );
                undo_changes_dense_hash_map_ast_local_constant_local_constant_change_log(
                    &mut self.locstants,
                    &self.local_changes,
                );
            } else {
                fold_constants(
                    &mut self.constants,
                    &mut self.variables,
                    &mut self.locstants,
                    self.builtins_fold,
                    self.builtins_fold_library_k,
                    self.options.library_member_constant_cb,
                    func_ref.body as *mut AstNode,
                    &mut *self.names,
                    &self.table_constants,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }

            cost
        }
    }
}
