use crate::enums::type_constant_folding::Type;
use crate::functions::analyze_builtins::analyze_builtins;
use crate::functions::fold_constants::fold_constants;
use crate::functions::undo_changes_constant_folding::undo_changes_dense_hash_map_ast_expr_constant_expr_constant_change_log;
use crate::functions::undo_changes_constant_folding_alt_b::undo_changes_dense_hash_map_ast_local_constant_local_constant_change_log;
use crate::records::compiler::Compiler;
use crate::records::constant::{Constant, ConstantData};
use crate::records::inline_arg::InlineArg;
use crate::records::inline_frame::InlineFrame;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_expr_varargs::AstExprVarargs;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat::AstStat;
use luaur_common::enums::luau_opcode::LuauOpcode;

const K_INVALID_REG: u8 = 255;
const K_DEFAULT_ALLOC_PC: u32 = !0u32;

impl Compiler {
    pub fn compile_inlined_call(
        &mut self,
        expr: *mut AstExprCall,
        func: *mut AstExprFunction,
        target: u8,
        target_count: u8,
    ) {
        unsafe {
            let mut rs = self.reg_scope_compiler();
            let _ = &mut rs;
            let old_locals = self.local_stack.len();
            let mut args: Vec<InlineArg> = Vec::new();
            args.reserve((*func).args.size);

            let func_args_size = (*func).args.size;
            let expr_args_size = (*expr).args.size;

            // evaluate all arguments; note that we don't emit code for constant arguments (relying on constant folding)
            let mut i = 0usize;
            while i < func_args_size {
                let var: *mut AstLocal = *(*func).args.data.add(i);
                let arg: *mut AstExpr = if i < expr_args_size {
                    *(*expr).args.data.add(i)
                } else {
                    core::ptr::null_mut()
                };

                if i + 1 == expr_args_size
                    && func_args_size > expr_args_size
                    && self.is_expr_mult_ret(arg)
                {
                    let tail: u32 = (func_args_size - expr_args_size) as u32 + 1;
                    let reg = self.alloc_reg(arg as *mut AstNode, tail);
                    let allocpc = (*self.bytecode).get_debug_pc();

                    let call = luaur_ast::rtti::ast_node_as::<AstExprCall>(arg as *mut AstNode);
                    if !call.is_null() {
                        self.compile_expr_call(call, reg, tail as u8, true, false);
                    } else {
                        let va =
                            luaur_ast::rtti::ast_node_as::<AstExprVarargs>(arg as *mut AstNode);
                        if !va.is_null() {
                            self.compile_expr_varargs(va, reg, tail as u8, false);
                        } else {
                            luaur_common::macros::luau_assert::LUAU_ASSERT!(
                                false,
                                "Unexpected expression type"
                            );
                        }
                    }

                    let mut j = i;
                    while j < func_args_size {
                        args.push(InlineArg {
                            local: *(*func).args.data.add(j),
                            reg: reg + (j - i) as u8,
                            value: Constant {
                                r#type: Type::Type_Unknown,
                                string_length: 0,
                                data: ConstantData { value_number: 0.0 },
                            },
                            allocpc,
                            init: core::ptr::null_mut(),
                        });
                        j += 1;
                    }
                    break;
                } else if {
                    let vv = self.variables.find(&var);
                    vv.map_or(false, |vv| vv.written)
                } {
                    let reg = self.alloc_reg(arg as *mut AstNode, 1u32);
                    let allocpc = (*self.bytecode).get_debug_pc();
                    if !arg.is_null() {
                        self.compile_expr_temp(arg, reg);
                    } else {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_LOADNIL, reg, 0, 0);
                    }
                    args.push(InlineArg {
                        local: var,
                        reg,
                        value: Constant {
                            r#type: Type::Type_Unknown,
                            string_length: 0,
                            data: ConstantData { value_number: 0.0 },
                        },
                        allocpc,
                        init: core::ptr::null_mut(),
                    });
                } else if arg.is_null() {
                    args.push(InlineArg {
                        local: var,
                        reg: K_INVALID_REG,
                        value: Constant {
                            r#type: Type::Type_Nil,
                            string_length: 0,
                            data: ConstantData { value_number: 0.0 },
                        },
                        allocpc: K_DEFAULT_ALLOC_PC,
                        init: core::ptr::null_mut(),
                    });
                } else if {
                    let cv = self.constants.find(&arg);
                    cv.map_or(false, |cv| cv.r#type != Type::Type_Unknown)
                } {
                    let cv = *self.constants.find(&arg).unwrap();
                    args.push(InlineArg {
                        local: var,
                        reg: K_INVALID_REG,
                        value: cv,
                        allocpc: K_DEFAULT_ALLOC_PC,
                        init: core::ptr::null_mut(),
                    });
                } else {
                    let le = self.get_expr_local(arg);
                    let lv_written = if !le.is_null() {
                        self.variables.find(&(*le).local).map(|v| v.written)
                    } else {
                        None
                    };
                    let reg: i32 = if !le.is_null() {
                        self.get_expr_local_reg(le as *mut AstExpr)
                    } else {
                        -1
                    };
                    if reg >= 0 && (lv_written.is_none() || lv_written == Some(false)) {
                        let lv_init = if !le.is_null() {
                            self.variables
                                .find(&(*le).local)
                                .map_or(core::ptr::null_mut(), |v| v.init)
                        } else {
                            core::ptr::null_mut()
                        };
                        args.push(InlineArg {
                            local: var,
                            reg: reg as u8,
                            value: Constant {
                                r#type: Type::Type_Unknown,
                                string_length: 0,
                                data: ConstantData { value_number: 0.0 },
                            },
                            allocpc: K_DEFAULT_ALLOC_PC,
                            init: lv_init,
                        });
                    } else {
                        let temp = self.alloc_reg(arg as *mut AstNode, 1u32);
                        let allocpc = (*self.bytecode).get_debug_pc();
                        self.compile_expr_temp(arg, temp);
                        args.push(InlineArg {
                            local: var,
                            reg: temp,
                            value: Constant {
                                r#type: Type::Type_Unknown,
                                string_length: 0,
                                data: ConstantData { value_number: 0.0 },
                            },
                            allocpc,
                            init: arg,
                        });
                    }
                }

                i += 1;
            }

            // evaluate extra expressions for side effects
            let mut k = func_args_size;
            while k < expr_args_size {
                let side = *(*expr).args.data.add(k);
                self.compile_expr_side(side);
                k += 1;
            }

            // apply all evaluated arguments to the compiler state
            for arg in &args {
                if arg.value.r#type == Type::Type_Unknown {
                    self.push_local(arg.local, arg.reg, arg.allocpc);
                    if !arg.init.is_null() {
                        if let Some(lv) = self.variables.find_mut(&arg.local) {
                            lv.init = arg.init;
                        }
                    }
                } else {
                    *self.locstants.get_or_insert(arg.local) = arg.value;
                }
            }

            self.inline_frames.push(InlineFrame {
                func,
                local_offset: old_locals,
                target,
                target_count,
                return_jumps: Vec::new(),
            });

            let func_body = (*func).body;

            {
                let ib = &mut self.inline_builtins as *mut _;
                analyze_builtins(
                    &mut *ib,
                    &self.globals,
                    &self.variables,
                    &self.options,
                    func_body as *mut AstNode,
                    &*self.names,
                );
            }

            if !self.inline_builtins.is_empty() {
                let entries: Vec<(*mut AstExprCall, i32)> =
                    self.inline_builtins.iter().map(|(k, v)| (*k, *v)).collect();
                for (call_expr, bfid) in entries {
                    let builtin = *self.builtins.get_or_insert(call_expr);
                    if bfid != builtin {
                        *self.inline_builtins_backup.get_or_insert(call_expr) = builtin;
                        *self.builtins.get_or_insert(call_expr) = bfid;
                    }
                }
                self.inline_builtins.clear();
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
                func_body as *mut AstNode,
                &mut *self.names,
                &self.table_constants,
                &mut self.expr_changes as *mut _,
                &mut self.local_changes as *mut _,
            );

            let mut terminates_early = false;
            let body_size = (*func_body).body.size;
            let mut bi = 0usize;
            while bi < body_size {
                let stat: *mut AstStat = *(*func_body).body.data.add(bi);
                self.compile_stat(stat);
                if self.always_terminates(stat) {
                    terminates_early = true;
                    let curr_frame = self.inline_frames.last_mut().unwrap();
                    if !curr_frame.return_jumps.is_empty() {
                        let last_jump = *curr_frame.return_jumps.last().unwrap();
                        if last_jump == (*self.bytecode).emit_label() - 1 {
                            (*self.bytecode).undo_emit(LuauOpcode::LOP_JUMP);
                            curr_frame.return_jumps.pop();
                        }
                    }
                    break;
                }
                bi += 1;
            }

            if !terminates_early {
                let mut t = 0usize;
                while t < target_count as usize {
                    (*self.bytecode).emit_abc(LuauOpcode::LOP_LOADNIL, target + t as u8, 0, 0);
                    t += 1;
                }
                self.close_locals(old_locals);
            }

            self.pop_locals(old_locals);
            let return_label = (*self.bytecode).emit_label();
            let rj = &mut self.inline_frames.last_mut().unwrap().return_jumps as *mut Vec<usize>;
            self.patch_jumps(expr as *mut AstNode, &mut *rj, return_label);
            self.inline_frames.pop();

            // clean up constant state for future inlining attempts
            let mut ci = 0usize;
            while ci < func_args_size {
                let local: *mut AstLocal = *(*func).args.data.add(ci);
                if let Some(var) = self.locstants.find_mut(&local) {
                    var.r#type = Type::Type_Unknown;
                }
                if let Some(lv) = self.variables.find_mut(&local) {
                    lv.init = core::ptr::null_mut();
                }
                ci += 1;
            }

            if !self.inline_builtins_backup.is_empty() {
                let entries: Vec<(*mut AstExprCall, i32)> = self
                    .inline_builtins_backup
                    .iter()
                    .map(|(k, v)| (*k, *v))
                    .collect();
                for (call_expr, bfid) in entries {
                    *self.builtins.get_or_insert(call_expr) = bfid;
                }
                self.inline_builtins_backup.clear();
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
                    func_body as *mut AstNode,
                    &mut *self.names,
                    &self.table_constants,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }
        }
    }
}
