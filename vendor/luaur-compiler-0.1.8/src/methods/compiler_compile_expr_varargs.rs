use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_varargs::AstExprVarargs;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_expr_varargs(
        &mut self,
        expr: *mut AstExprVarargs,
        target: u8,
        target_count: u8,
        mult_ret: bool,
    ) {
        LUAU_ASSERT!(target_count < 255);
        LUAU_ASSERT!(!mult_ret || u32::from(target) + u32::from(target_count) == self.reg_top);

        self.set_debug_line_ast_node(expr as *mut luaur_ast::records::ast_node::AstNode);

        unsafe {
            let bytecode = &mut *self.bytecode;
            bytecode.emit_abc(
                LuauOpcode::LOP_GETVARARGS,
                target,
                if mult_ret {
                    0
                } else {
                    target_count.wrapping_add(1)
                },
                0,
            );
        }
    }
}
