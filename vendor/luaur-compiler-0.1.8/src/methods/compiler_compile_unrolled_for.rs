use crate::enums::type_compiler::Type as LoopJumpType;
use crate::enums::type_constant_folding::Type;
use crate::functions::fold_constants::fold_constants;
use crate::functions::undo_changes_constant_folding::undo_changes_dense_hash_map_ast_expr_constant_expr_constant_change_log;
use crate::functions::undo_changes_constant_folding_alt_b::undo_changes_dense_hash_map_ast_local_constant_local_constant_change_log;
use crate::records::compiler::Compiler;
use crate::records::constant::{Constant, ConstantData};
use crate::records::loop_jump::LoopJump;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::ast_stat_for::AstStatFor;

impl Compiler {
    pub fn compile_unrolled_for(
        &mut self,
        stat: *mut AstStatFor,
        trip_count: i32,
        from: f64,
        step: f64,
    ) {
        unsafe {
            let stat_ref = &*stat;
            let old_locals = self.local_stack.len();
            let old_jumps = self.loop_jumps.len();

            self.loops.push(crate::records::r#loop::Loop {
                local_offset: old_locals,
                local_offset_continue: old_locals,
                continue_used: core::ptr::null_mut(),
            });

            let record_changes = luaur_common::FFlag::LuauCompilePropagateTableProps2.get()
                && luaur_common::FFlag::LuauCompileFoldOptimize.get();

            if record_changes {
                self.expr_changes.clear();
                self.local_changes.clear();
            }

            for iv in 0..trip_count {
                *self.locstants.get_or_insert(stat_ref.var) = Constant {
                    r#type: Type::Type_Number,
                    string_length: 0,
                    data: ConstantData {
                        value_number: from + f64::from(iv) * step,
                    },
                };

                fold_constants(
                    &mut self.constants,
                    &mut self.variables,
                    &mut self.locstants,
                    self.builtins_fold,
                    self.builtins_fold_library_k,
                    self.options.library_member_constant_cb,
                    stat_ref.body as *mut AstNode,
                    &mut *self.names,
                    &self.table_constants,
                    if record_changes && iv == 0 {
                        &mut self.expr_changes as *mut _
                    } else {
                        core::ptr::null_mut()
                    },
                    if record_changes && iv == 0 {
                        &mut self.local_changes as *mut _
                    } else {
                        core::ptr::null_mut()
                    },
                );

                let iter_jumps = self.loop_jumps.len();
                self.compile_stat(stat_ref.body as *mut AstStat);

                let cont_label = (*self.bytecode).emit_label();

                for i in iter_jumps..self.loop_jumps.len() {
                    if self.loop_jumps[i].r#type == LoopJumpType::Continue {
                        self.patch_jump(stat as *mut AstNode, self.loop_jumps[i].label, cont_label);
                    }
                }
            }

            let end_label = (*self.bytecode).emit_label();

            for i in old_jumps..self.loop_jumps.len() {
                if self.loop_jumps[i].r#type == LoopJumpType::Break {
                    self.patch_jump(stat as *mut AstNode, self.loop_jumps[i].label, end_label);
                }
            }

            self.loop_jumps.resize(
                old_jumps,
                LoopJump {
                    r#type: LoopJumpType::Break,
                    label: 0,
                },
            );

            self.loops.pop();

            self.locstants.get_or_insert(stat_ref.var).r#type = Type::Type_Unknown;

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
                    stat_ref.body as *mut AstNode,
                    &mut *self.names,
                    &self.table_constants,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }
        }
    }
}
