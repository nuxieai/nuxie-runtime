use crate::records::compiler::Compiler;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_stat_for_in::AstStatForIn;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_stat_for_in(&mut self, stat: *mut AstStatForIn) {
        unsafe {
            let stat_ref = &*stat;
            let mut rs = self.reg_scope_compiler();

            let old_locals = self.local_stack.len();
            let old_jumps = self.loop_jumps.len();

            self.loops.push(crate::records::r#loop::Loop {
                local_offset: old_locals,
                local_offset_continue: old_locals,
                continue_used: core::ptr::null_mut(),
            });
            self.has_loops = true;

            let regs = self.alloc_reg(stat as *mut _, 3);

            self.compile_expr_list_temp(&stat_ref.values, regs, 3, true);

            let vars = self.alloc_reg(stat as *mut _, core::cmp::max(stat_ref.vars.size as u32, 2));
            LUAU_ASSERT!(vars == regs + 3);
            let vars_alloc_pc = (*self.bytecode).get_debug_pc();

            let mut skip_op = LuauOpcode::LOP_FORGPREP;

            if self.options.optimization_level >= 1 && stat_ref.vars.size <= 2 {
                if stat_ref.values.size == 1
                    && luaur_ast::rtti::ast_node_is::<luaur_ast::records::ast_expr_call::AstExprCall>(
                        *stat_ref.values.data as *mut luaur_ast::records::ast_node::AstNode,
                    )
                {
                    // C++ passes the CALL's `func` (the `ipairs`/`pairs` reference),
                    // NOT the call expression itself, to getBuiltin. The model passed
                    // the whole `ipairs(t)` call, so it never matched the builtin and
                    // the loop fell back to generic FORGPREP.
                    let call = luaur_ast::rtti::ast_node_as::<
                        luaur_ast::records::ast_expr_call::AstExprCall,
                    >(
                        *stat_ref.values.data as *mut luaur_ast::records::ast_node::AstNode,
                    );
                    let builtin = crate::functions::get_builtin::get_builtin(
                        (*call).func,
                        &self.globals,
                        &self.variables,
                    );

                    if builtin.is_global("ipairs") {
                        skip_op = LuauOpcode::LOP_FORGPREP_INEXT;
                    } else if builtin.is_global("pairs") {
                        skip_op = LuauOpcode::LOP_FORGPREP_NEXT;
                    }
                } else if stat_ref.values.size == 2 {
                    let builtin = crate::functions::get_builtin::get_builtin(
                        *stat_ref.values.data,
                        &self.globals,
                        &self.variables,
                    );

                    if builtin.is_global("next") {
                        skip_op = LuauOpcode::LOP_FORGPREP_NEXT;
                    }
                }
            }

            let skip_label = (*self.bytecode).emit_label();

            (*self.bytecode).emit_ad(skip_op, regs, 0);

            let loop_label = (*self.bytecode).emit_label();

            for i in 0..stat_ref.vars.size {
                self.push_local(
                    *stat_ref.vars.data.add(i as usize),
                    (vars + i as u8) as u8,
                    vars_alloc_pc,
                );
            }

            self.compile_stat(stat_ref.body as *mut _);

            self.close_locals(old_locals);
            self.pop_locals(old_locals);

            self.set_debug_line_ast_node(stat as *mut _);

            let cont_label = (*self.bytecode).emit_label();
            let back_label = (*self.bytecode).emit_label();

            (*self.bytecode).emit_ad(LuauOpcode::LOP_FORGLOOP, regs, 0);
            (*self.bytecode).emit_aux(
                if skip_op == LuauOpcode::LOP_FORGPREP_INEXT {
                    0x80000000
                } else {
                    0
                } | stat_ref.vars.size as u32,
            );

            let end_label = (*self.bytecode).emit_label();

            self.patch_jump(stat as *mut _, skip_label, back_label);
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
