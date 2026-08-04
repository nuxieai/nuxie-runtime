use crate::records::compiler::Compiler;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_stat_for::AstStatFor;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_stat_for(&mut self, stat: *mut AstStatFor) {
        unsafe {
            let stat_ref = &*stat;
            let mut rs = self.reg_scope_compiler();

            if self.options.optimization_level >= 2
                && self.is_constant(stat_ref.to)
                && self.is_constant(stat_ref.from)
                && (stat_ref.step.is_null() || self.is_constant(stat_ref.step))
            {
                // C++ passes FInt::LuauCompileLoopUnrollThreshold / ...MaxBoost; the
                // hardcoded 100/100 ignored both the configured defaults and test
                // ScopedFastInt overrides, so loops with more iterations than the
                // threshold (e.g. for i=1,100 under threshold 25) wrongly unrolled.
                if self.try_compile_unrolled_for(
                    stat,
                    luaur_common::FInt::LuauCompileLoopUnrollThreshold.get(),
                    luaur_common::FInt::LuauCompileLoopUnrollThresholdMaxBoost.get(),
                ) {
                    return;
                }
            }

            let old_locals = self.local_stack.len();
            let old_jumps = self.loop_jumps.len();

            self.loops.push(crate::records::r#loop::Loop {
                local_offset: old_locals,
                local_offset_continue: old_locals,
                continue_used: core::ptr::null_mut(),
            });
            self.has_loops = true;

            let regs = self.alloc_reg(stat as *mut _, 3);
            let varregallocpc = (*self.bytecode).get_debug_pc();
            let mut varreg = regs + 2;

            if let Some(il) = self.variables.find(&stat_ref.var) {
                if il.written {
                    varreg = self.alloc_reg(stat as *mut _, 1);
                }
            }

            self.compile_expr_temp(stat_ref.from, regs + 2);
            self.compile_expr_temp(stat_ref.to, regs + 0);

            if !stat_ref.step.is_null() {
                self.compile_expr_temp(stat_ref.step, regs + 1);
            } else {
                (*self.bytecode).emit_abc(LuauOpcode::LOP_LOADN, regs + 1, 1, 0);
            }

            let for_label = (*self.bytecode).emit_label();
            (*self.bytecode).emit_ad(LuauOpcode::LOP_FORNPREP, regs, 0);
            let loop_label = (*self.bytecode).emit_label();

            if varreg != regs + 2 {
                (*self.bytecode).emit_abc(LuauOpcode::LOP_MOVE, varreg, regs + 2, 0);
            }

            self.push_local(stat_ref.var, varreg, varregallocpc);
            self.compile_stat(stat_ref.body as *mut _);

            self.close_locals(old_locals);
            self.pop_locals(old_locals);
            self.set_debug_line_ast_node(stat as *mut _);

            let cont_label = (*self.bytecode).emit_label();
            let back_label = (*self.bytecode).emit_label();
            (*self.bytecode).emit_ad(LuauOpcode::LOP_FORNLOOP, regs, 0);
            let end_label = (*self.bytecode).emit_label();

            self.patch_jump(stat as *mut _, for_label, end_label);
            self.patch_jump(stat as *mut _, back_label, loop_label);

            self.patch_loop_jumps(stat as *mut _, old_jumps, end_label, cont_label);
            self.loop_jumps.resize(
                old_jumps,
                crate::records::loop_jump::LoopJump {
                    r#type: crate::enums::type_compiler::Type::Break,
                    label: 0,
                },
            );

            self.loops.pop();
        }
    }
}
