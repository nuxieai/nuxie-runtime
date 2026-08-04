use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_expr_fastcall_n(
        &mut self,
        expr: *mut AstExprCall,
        target: u8,
        target_count: u8,
        target_top: bool,
        mult_ret: bool,
        regs: u8,
        bfid: i32,
        bf_k: i32,
    ) {
        let expr_ref = unsafe { &*expr };
        LUAU_ASSERT!(!expr_ref.self_);
        LUAU_ASSERT!(expr_ref.args.size >= 1);
        LUAU_ASSERT!(expr_ref.args.size <= 3);
        LUAU_ASSERT!(if bfid
            == luaur_common::enums::luau_builtin_function::LuauBuiltinFunction::LBF_BIT32_EXTRACTK
                as i32
        {
            bf_k >= 0
        } else {
            bf_k < 0
        });
        LUAU_ASSERT!(target_count < 255);

        let opc = if expr_ref.args.size == 1 {
            LuauOpcode::LOP_FASTCALL1
        } else if bf_k >= 0
            || (expr_ref.args.size == 2 && self.is_constant(unsafe { *expr_ref.args.data.add(1) }))
        {
            LuauOpcode::LOP_FASTCALL2K
        } else if expr_ref.args.size == 2 {
            LuauOpcode::LOP_FASTCALL2
        } else {
            LuauOpcode::LOP_FASTCALL3
        };

        let mut args = [0u32; 3];
        for i in 0..expr_ref.args.size {
            let arg_expr = unsafe { *expr_ref.args.data.add(i) };
            if i > 0 && opc == LuauOpcode::LOP_FASTCALL2K {
                let cid = self.get_constant_index(arg_expr);
                if cid < 0 {
                    let location = unsafe { (*arg_expr).base.location };
                    CompileError::raise(
                        &location,
                        core::format_args!("Exceeded constant limit; simplify the code to compile"),
                    );
                }
                args[i] = cid as u32;
            } else if let Some(reg) = {
                let r = self.get_expr_local_reg(arg_expr);
                if r >= 0 {
                    Some(r)
                } else {
                    None
                }
            } {
                args[i] = reg as u32;
            } else {
                args[i] = (regs as u32) + 1 + (i as u32);
                self.compile_expr_temp_top(arg_expr, args[i] as u8);
            }
        }

        let bytecode = unsafe { &mut *self.bytecode };
        let fastcall_label = bytecode.emit_label();

        bytecode.emit_abc(opc, bfid as u8, args[0] as u8, 0);

        if opc == LuauOpcode::LOP_FASTCALL3 {
            LUAU_ASSERT!(bf_k < 0);
            bytecode.emit_aux(args[1] | (args[2] << 8));
        } else if opc != LuauOpcode::LOP_FASTCALL1 {
            bytecode.emit_aux(if bf_k >= 0 { bf_k as u32 } else { args[1] });
        }

        for i in 0..expr_ref.args.size {
            if i > 0 && opc == LuauOpcode::LOP_FASTCALL2K {
                self.emit_load_k((regs + 1 + i as u8), args[i] as i32);
            } else if args[i] != (regs as u32) + 1 + (i as u32) {
                bytecode.emit_abc(LuauOpcode::LOP_MOVE, (regs + 1 + i as u8), args[i] as u8, 0);
            }
        }

        self.compile_expr_temp(expr_ref.func, regs);

        let call_label = bytecode.emit_label();

        if !bytecode.patch_skip_c(fastcall_label, call_label) {
            let location = unsafe { (*expr_ref.func).base.location };
            CompileError::raise(
                &location,
                core::format_args!("Exceeded jump distance limit; simplify the code to compile"),
            );
        }

        bytecode.emit_abc(
            LuauOpcode::LOP_CALL,
            regs,
            (expr_ref.args.size + 1) as u8,
            if mult_ret { 0 } else { target_count + 1 },
        );

        if !target_top {
            for i in 0..target_count {
                bytecode.emit_abc(LuauOpcode::LOP_MOVE, target + i, regs + i, 0);
            }
        }
    }
}
