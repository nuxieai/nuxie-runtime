use crate::records::compiler::Compiler;
use crate::records::inline_frame::InlineFrame;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat_return::AstStatReturn;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_inline_return(&mut self, stat: *mut AstStatReturn, _fallthrough: bool) {
        self.set_debug_line_ast_node(stat as *mut AstNode);

        let frame = self
            .inline_frames
            .last()
            .expect("inline_frames must not be empty")
            .clone();

        unsafe {
            let stat_ref = &*stat;
            self.compile_expr_list_temp(&stat_ref.list, frame.target, frame.target_count, false);
        }

        self.close_locals(frame.local_offset);

        let jump_label = unsafe { (*self.bytecode).emit_label() };
        unsafe {
            (*self.bytecode).emit_ad(LuauOpcode::LOP_JUMP, 0, 0);
        }

        self.inline_frames
            .last_mut()
            .expect("inline_frames must not be empty")
            .return_jumps
            .push(jump_label);
    }
}
