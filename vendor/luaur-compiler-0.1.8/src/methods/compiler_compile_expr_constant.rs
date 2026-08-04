use crate::enums::type_constant_folding::Type;
use crate::functions::sref_compiler_alt_c::sref_ast_array_c_char;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_common::enums::luau_opcode::LuauOpcode;

use core::ffi::c_char;

impl Compiler {
    pub fn compile_expr_constant(&mut self, node: *mut AstExpr, cv: &Constant, target: u8) {
        match cv.r#type {
            Type::Type_Nil => unsafe {
                (*self.bytecode).emit_abc(LuauOpcode::LOP_LOADNIL, target, 0, 0);
            },
            Type::Type_Boolean => {
                let b = unsafe { cv.data.value_boolean };
                unsafe {
                    (*self.bytecode).emit_abc(LuauOpcode::LOP_LOADB, target, b as u8, 0);
                }
            }
            Type::Type_Number => {
                let d = unsafe { cv.data.value_number };

                let fits_i16 = d >= (i16::MIN as f64)
                    && d <= (i16::MAX as f64)
                    && (d as i16 as f64) == d
                    && !(d == 0.0 && d.is_sign_negative());

                if fits_i16 {
                    unsafe {
                        (*self.bytecode).emit_ad(LuauOpcode::LOP_LOADN, target, d as i16);
                    }
                } else {
                    let cid = unsafe { (*self.bytecode).add_constant_number(d) };
                    if cid < 0 {
                        let location = unsafe { (*node).base.location };
                        CompileError::raise(
                            &location,
                            format_args!("Exceeded constant limit; simplify the code to compile"),
                        );
                    }
                    self.emit_load_k(target, cid);
                }
            }
            Type::Type_Integer => {
                let l = unsafe { cv.data.value_integer64 };
                let cid = unsafe { (*self.bytecode).add_constant_integer(l) };
                if cid < 0 {
                    let location = unsafe { (*node).base.location };
                    CompileError::raise(
                        &location,
                        format_args!("Exceeded constant limit; simplify the code to compile"),
                    );
                }
                self.emit_load_k(target, cid);
            }
            Type::Type_Vector => {
                let x = unsafe { cv.data.value_vector[0] };
                let y = unsafe { cv.data.value_vector[1] };
                let z = unsafe { cv.data.value_vector[2] };
                let w = unsafe { cv.data.value_vector[3] };

                let cid = unsafe { (*self.bytecode).add_constant_vector(x, y, z, w) };
                if cid < 0 {
                    let location = unsafe { (*node).base.location };
                    CompileError::raise(
                        &location,
                        format_args!("Exceeded constant limit; simplify the code to compile"),
                    );
                }
                self.emit_load_k(target, cid);
            }
            Type::Type_String => {
                let s = cv.get_string();
                let cid = unsafe { (*self.bytecode).add_constant_string(sref_ast_array_c_char(s)) };
                if cid < 0 {
                    let location = unsafe { (*node).base.location };
                    CompileError::raise(
                        &location,
                        format_args!("Exceeded constant limit; simplify the code to compile"),
                    );
                }
                self.emit_load_k(target, cid);
            }
            _ => {
                luaur_common::macros::luau_assert::LUAU_ASSERT!(false);
            }
        }
    }
}
