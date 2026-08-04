use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat_while::AstStatWhile;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_stat_while(&mut self, stat: *mut AstStatWhile) {
        unsafe {
            let stat_ref = &*stat;

            // Optimization: condition is always false => there's no loop!
            if self.is_constant_false(stat_ref.condition) {
                return;
            }

            let old_jumps = self.loop_jumps.len();
            let old_locals = self.local_stack.len();

            self.loops.push(crate::records::r#loop::Loop {
                local_offset: old_locals,
                local_offset_continue: old_locals,
                continue_used: core::ptr::null_mut(),
            });
            self.has_loops = true;

            let loop_label = (*self.bytecode).emit_label();

            let mut else_jump = Vec::new();
            self.compile_condition_value(
                stat_ref.condition,
                core::ptr::null(),
                &mut else_jump,
                false,
            );

            self.compile_stat(stat_ref.body as *mut luaur_ast::records::ast_stat::AstStat);

            let cont_label = (*self.bytecode).emit_label();
            let back_label = (*self.bytecode).emit_label();

            self.set_debug_line_ast_node(stat as *mut AstNode);

            // Note: this is using JUMPBACK, not JUMP, since JUMPBACK is interruptable and we want all loops to have at least one interruptable instruction
            (*self.bytecode).emit_ad(LuauOpcode::LOP_JUMPBACK, 0, 0);

            let end_label = (*self.bytecode).emit_label();

            self.patch_jump(stat as *mut AstNode, back_label, loop_label);
            self.patch_jumps(stat as *mut AstNode, &mut else_jump, end_label);

            self.patch_loop_jumps(stat as *mut AstNode, old_jumps, end_label, cont_label);
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
