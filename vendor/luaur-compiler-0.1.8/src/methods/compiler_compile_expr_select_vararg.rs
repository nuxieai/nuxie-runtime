use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_varargs::AstExprVarargs;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_expr_select_vararg(
        &mut self,
        expr: *mut AstExprCall,
        target: u8,
        target_count: u8,
        target_top: bool,
        mult_ret: bool,
        regs: u8,
    ) {
        LUAU_ASSERT!(target_count == 1);
        let expr_ref = unsafe { &*expr };
        LUAU_ASSERT!(!expr_ref.self_);
        LUAU_ASSERT!(expr_ref.args.size == 2);
        let arg = unsafe { *expr_ref.args.data.add(0) };
        let arg_varargs = unsafe { *expr_ref.args.data.add(1) };
        LUAU_ASSERT!(
            !unsafe { rtti::ast_node_as::<AstExprVarargs>(arg_varargs as *mut AstNode) }.is_null()
        );

        let argreg: u8;
        let reg = self.get_expr_local_reg(arg);
        if reg >= 0 {
            argreg = reg as u8;
        } else {
            argreg = regs + 1;
            self.compile_expr_temp_top(arg, argreg);
        }

        let fastcall_label = unsafe { (*self.bytecode).emit_label() };

        unsafe {
            (*self.bytecode).emit_abc(
                LuauOpcode::LOP_FASTCALL1,
                luaur_common::enums::luau_builtin_function::LuauBuiltinFunction::LBF_SELECT_VARARG
                    as u8,
                argreg,
                0,
            );
        }

        self.compile_expr_temp(expr_ref.func, regs);

        if argreg != regs + 1 {
            unsafe {
                (*self.bytecode).emit_abc(LuauOpcode::LOP_MOVE, regs + 1, argreg, 0);
            }
        }

        unsafe {
            (*self.bytecode).emit_abc(LuauOpcode::LOP_GETVARARGS, regs + 2, 0, 0);
        }

        let call_label = unsafe { (*self.bytecode).emit_label() };
        if !unsafe { (*self.bytecode).patch_skip_c(fastcall_label, call_label) } {
            CompileError::raise(
                unsafe { &(*expr_ref.func).base.location },
                core::format_args!("Exceeded jump distance limit; simplify the code to compile"),
            );
        }

        unsafe {
            (*self.bytecode).emit_abc(
                LuauOpcode::LOP_CALL,
                regs,
                0,
                if mult_ret { 0 } else { target_count + 1 },
            );
        }

        if !target_top {
            for i in 0..target_count {
                unsafe {
                    (*self.bytecode).emit_abc(LuauOpcode::LOP_MOVE, target + i, regs + i, 0);
                }
            }
        }
    }
}
