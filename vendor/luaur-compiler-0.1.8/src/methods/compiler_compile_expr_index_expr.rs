use crate::enums::type_constant_folding::Type;
use crate::functions::sref_compiler_alt_c::sref_ast_array_c_char;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr_index_expr::AstExprIndexExpr;
use luaur_bytecode::methods::bytecode_builder_get_string_hash::bytecode_builder_get_string_hash;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_expr_index_expr(&mut self, expr: *mut AstExprIndexExpr, target: u8) {
        unsafe {
            let expr_ref = &*expr;
            let mut rs = self.reg_scope_compiler();
            let cv = self.get_constant(expr_ref.index);

            if cv.r#type == Type::Type_Number
                && cv.data.value_number >= 1.0
                && cv.data.value_number <= 256.0
                && (cv.data.value_number as i32) as f64 == cv.data.value_number
            {
                let i = (cv.data.value_number as i32 - 1) as u8;
                let rt = self.compile_expr_auto(expr_ref.expr, &mut rs);
                self.set_debug_line_location(&(*expr_ref.index).base.location);
                (*self.bytecode).emit_abc(LuauOpcode::LOP_GETTABLEN, target, rt, i);
                self.hint_temporary_expr_reg_type(expr_ref.expr, rt as i32, LuauBytecodeType(4), 1);
            } else if cv.r#type == Type::Type_String {
                let iname = sref_ast_array_c_char(cv.get_string());
                let cid = (*self.bytecode).add_constant_string(iname);
                if cid < 0 {
                    crate::records::compile_error::CompileError::raise(
                        &expr_ref.base.base.location,
                        format_args!("Exceeded constant limit; simplify the code to compile"),
                    );
                }
                let rt = self.compile_expr_auto(expr_ref.expr, &mut rs);
                self.set_debug_line_location(&(*expr_ref.index).base.location);
                (*self.bytecode).emit_abc(
                    LuauOpcode::LOP_GETTABLEKS,
                    target,
                    rt,
                    bytecode_builder_get_string_hash(iname) as u8,
                );
                (*self.bytecode).emit_aux(cid as u32);
                self.hint_temporary_expr_reg_type(expr_ref.expr, rt as i32, LuauBytecodeType(4), 2);
            } else {
                let rt = self.compile_expr_auto(expr_ref.expr, &mut rs);
                let ri = self.compile_expr_auto(expr_ref.index, &mut rs);
                (*self.bytecode).emit_abc(LuauOpcode::LOP_GETTABLE, target, rt, ri);
                self.hint_temporary_expr_reg_type(expr_ref.expr, rt as i32, LuauBytecodeType(4), 1);
                self.hint_temporary_expr_reg_type(
                    expr_ref.index,
                    ri as i32,
                    LuauBytecodeType(2),
                    1,
                );
            }
        }
    }
}
