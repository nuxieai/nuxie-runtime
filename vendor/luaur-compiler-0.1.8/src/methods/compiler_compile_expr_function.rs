use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_common::enums::luau_capture_type::LuauCaptureType;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_expr_function(&mut self, expr: *mut AstExprFunction, target: u8) {
        let f = self.functions.find(&expr).unwrap();
        let fid = f.id;
        let upvals = f.upvals.clone();
        let pid = unsafe { (*self.bytecode).add_child_function(fid) };
        if pid < 0 {
            unsafe {
                crate::records::compile_error::CompileError::raise(
                    &(*expr).base.base.location,
                    format_args!("Exceeded closure limit"),
                );
            }
        }
        self.captures.clear();
        for uv in upvals {
            let reg = self.get_local_reg(uv);
            if reg >= 0 {
                let ul = self.variables.find(&uv);
                let immutable = ul.map_or(true, |ul| !ul.written);
                self.captures.push(crate::records::capture::Capture {
                    r#type: if immutable {
                        LuauCaptureType::LCT_VAL
                    } else {
                        LuauCaptureType::LCT_REF
                    },
                    data: reg as u8,
                });
            } else if let Some(uc) = self.locstants.find(&uv).copied() {
                let reg = self.alloc_reg(expr as *mut _, 1);
                self.compile_expr_constant(expr as *mut _, &uc, reg);
                self.captures.push(crate::records::capture::Capture {
                    r#type: LuauCaptureType::LCT_VAL,
                    data: reg,
                });
            } else {
                let uid = self.get_upval(uv);
                self.captures.push(crate::records::capture::Capture {
                    r#type: LuauCaptureType::LCT_UPVAL,
                    data: uid,
                });
            }
        }
        let mut shared = -1i16;
        if self.options.optimization_level >= 1
            && self.should_share_closure(expr)
            && !self.setfenv_used
        {
            let cid = unsafe { (*self.bytecode).add_constant_closure(fid) };
            if cid >= 0 && cid < 32768 {
                shared = cid as i16;
            }
        }
        unsafe {
            if shared >= 0 {
                (*self.bytecode).emit_ad(LuauOpcode::LOP_DUPCLOSURE, target, shared);
            } else {
                (*self.bytecode).emit_ad(LuauOpcode::LOP_NEWCLOSURE, target, pid);
            }
            for c in &self.captures {
                (*self.bytecode).emit_abc(LuauOpcode::LOP_CAPTURE, c.r#type as u8, c.data, 0);
            }
        }
    }
}
