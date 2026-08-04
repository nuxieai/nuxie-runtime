use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_varargs::AstExprVarargs;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_expr_temp_n(
        &mut self,
        node: *mut AstExpr,
        target: u8,
        target_count: u8,
        target_top: bool,
    ) {
        LUAU_ASSERT!(!target_top || u32::from(target) + u32::from(target_count) == self.reg_top);

        if target_count == 255 {
            let location = unsafe { (*node).base.location };
            crate::records::compile_error::CompileError::raise(
                &location,
                core::format_args!("Exceeded result count limit; simplify the code to compile"),
            );
        }

        let expr_call = unsafe { rtti::ast_node_as::<AstExprCall>(node as *mut AstNode) };
        if !expr_call.is_null() {
            self.compile_expr_call(expr_call, target, target_count, target_top, false);
            return;
        }

        let expr_varargs = unsafe { rtti::ast_node_as::<AstExprVarargs>(node as *mut AstNode) };
        if !expr_varargs.is_null() {
            self.compile_expr_varargs(expr_varargs, target, target_count, false);
            return;
        }

        self.compile_expr_temp(node, target);

        for i in 1..target_count {
            unsafe {
                let bytecode = &mut *self.bytecode;
                bytecode.emit_abc(LuauOpcode::LOP_LOADNIL, target.wrapping_add(i), 0, 0);
            }
        }
    }
}
