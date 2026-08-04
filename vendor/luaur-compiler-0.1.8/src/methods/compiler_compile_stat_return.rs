use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_stat_return::AstStatReturn;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_stat_return(&mut self, stat: *mut AstStatReturn) {
        unsafe {
            let stat_ref = &*stat;
            if stat_ref.list.size >= 255 {
                CompileError::raise(
                    &stat_ref.base.base.location,
                    format_args!("Exceeded return count limit; simplify the code to compile"),
                );
            }

            let mut rs = self.reg_scope_compiler();
            let mut temp = 0u8;
            let mut consecutive = false;
            let mut mult_ret = false;

            if stat_ref.list.size > 0 {
                let reg = self.get_expr_local_reg(*stat_ref.list.data);
                if reg >= 0 {
                    temp = reg as u8;
                    consecutive = true;
                    for i in 1..stat_ref.list.size {
                        if self.get_expr_local_reg(*stat_ref.list.data.add(i))
                            != (temp as i32 + i as i32)
                        {
                            consecutive = false;
                            break;
                        }
                    }
                }
            }

            if !consecutive && stat_ref.list.size > 0 {
                temp = self.alloc_reg(stat as *mut _, stat_ref.list.size as u32);
                for i in 0..stat_ref.list.size {
                    if i + 1 == stat_ref.list.size {
                        mult_ret = self
                            .compile_expr_temp_mult_ret(*stat_ref.list.data.add(i), temp + i as u8);
                    } else {
                        self.compile_expr_temp_top(*stat_ref.list.data.add(i), temp + i as u8);
                    }
                }
            }

            self.close_locals(0);
            (*self.bytecode).emit_abc(
                LuauOpcode::LOP_RETURN,
                temp,
                if mult_ret {
                    0
                } else {
                    (stat_ref.list.size + 1) as u8
                },
                0,
            );
        }
    }
}
