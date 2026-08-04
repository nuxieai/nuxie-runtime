use crate::enums::type_constant_folding::Type;
use crate::functions::escape_and_append::escapeAndAppend;
use crate::functions::sref_compiler::sref_ast_name;
use crate::functions::sref_compiler_alt_c::sref_ast_array_c_char;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use crate::records::reg_scope::RegScope;
use alloc::vec::Vec;
use core::ffi::c_char;
use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_expr_interp_string::AstExprInterpString;
use luaur_ast::records::ast_name::AstName;
use luaur_bytecode::methods::bytecode_builder_get_string_hash::bytecode_builder_get_string_hash;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_expr_interp_string(
        &mut self,
        expr: *mut AstExprInterpString,
        target: u8,
        target_temp: bool,
    ) {
        unsafe {
            let expr_ref = &*expr;
            let mut format_capacity = 0;
            for string in expr_ref.strings.iter() {
                format_capacity +=
                    (*string).size + (*string).iter().filter(|&&c| c == b'%' as i8).count();
            }

            let mut skipped_sub_expr = 0;
            for i in 0..expr_ref.expressions.size {
                let sub_expr = *expr_ref.expressions.data.add(i);
                if let Some(c) = self.constants.find(&sub_expr) {
                    if c.r#type == Type::Type_String {
                        format_capacity += c.string_length as usize
                            + c.get_string().iter().filter(|&&c| c == b'%' as i8).count();
                        skipped_sub_expr += 1;
                    } else {
                        format_capacity += 2;
                    }
                } else {
                    format_capacity += 2;
                }
            }

            let mut format_string = Vec::with_capacity(format_capacity);
            for i in 0..expr_ref.strings.size {
                let string = *expr_ref.strings.data.add(i);
                escapeAndAppend(&mut format_string, string.data, string.size);
                if i < expr_ref.expressions.size {
                    let sub_expr = *expr_ref.expressions.data.add(i);
                    if let Some(c) = self.constants.find(&sub_expr) {
                        if c.r#type == Type::Type_String {
                            escapeAndAppend(
                                &mut format_string,
                                c.get_string().data,
                                c.string_length as usize,
                            );
                        } else {
                            format_string.extend_from_slice(b"%*");
                        }
                    } else {
                        format_string.extend_from_slice(b"%*");
                    }
                }
            }

            let format_string_index = if format_string.is_empty() {
                let interned = (*self.names).get_or_add(c"".as_ptr(), 0);
                (*self.bytecode).add_constant_string(sref_ast_name(interned))
            } else {
                let interned = (*self.names)
                    .get_or_add(format_string.as_ptr() as *const c_char, format_string.len());
                let format_string_array = AstArray {
                    data: interned.value as *mut c_char,
                    size: format_string.len(),
                };
                (*self.bytecode).add_constant_string(sref_ast_array_c_char(format_string_array))
            };

            if format_string_index < 0 {
                CompileError::raise(
                    &expr_ref.base.base.location,
                    format_args!("Exceeded constant limit; simplify the code to compile"),
                );
            }

            let mut rs = self.reg_scope_compiler();
            let reg_count = 2 + expr_ref.expressions.size - skipped_sub_expr;
            let target_top = luaur_common::FFlag::LuauCompileStringInterpTargetTop.get()
                && target_temp
                && target as u32 == self.reg_top - 1;
            let base_reg = if target_top {
                self.alloc_reg(expr as *mut _, (reg_count - 1) as u32) - 1
            } else {
                self.alloc_reg(expr as *mut _, reg_count as u32)
            };

            self.emit_load_k(base_reg, format_string_index);

            let mut skipped = 0;
            for i in 0..expr_ref.expressions.size {
                let sub_expr = *expr_ref.expressions.data.add(i);
                if self
                    .constants
                    .find(&sub_expr)
                    .map_or(true, |c| c.r#type != Type::Type_String)
                {
                    self.compile_expr_temp_top(sub_expr, base_reg + 2 + i as u8 - skipped as u8);
                } else {
                    skipped += 1;
                }
            }

            let format_method = sref_ast_name(AstName::ast_name_c_char(c"format".as_ptr()));
            let format_method_index = (*self.bytecode).add_constant_string(format_method);
            if format_method_index < 0 {
                CompileError::raise(
                    &expr_ref.base.base.location,
                    format_args!("Exceeded constant limit; simplify the code to compile"),
                );
            }

            (*self.bytecode).emit_abc(
                LuauOpcode::LOP_NAMECALL,
                base_reg,
                base_reg,
                bytecode_builder_get_string_hash(format_method) as u8,
            );
            (*self.bytecode).emit_aux(format_method_index as u32);
            (*self.bytecode).emit_abc(
                LuauOpcode::LOP_CALL,
                base_reg,
                (expr_ref.expressions.size + 2 - skipped_sub_expr) as u8,
                2,
            );
            if target != base_reg {
                (*self.bytecode).emit_abc(LuauOpcode::LOP_MOVE, target, base_reg, 0);
            }
        }
    }
}
