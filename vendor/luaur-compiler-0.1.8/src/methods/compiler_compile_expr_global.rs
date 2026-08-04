use crate::functions::sref_compiler::sref_ast_name;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_bytecode::methods::bytecode_builder_get_string_hash::bytecode_builder_get_string_hash;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_expr_global(&mut self, expr: *mut AstExprGlobal, target: u8) {
        unsafe {
            let name = sref_ast_name((*expr).name);
            let cid = (*self.bytecode).add_constant_string(name);
            if cid < 0 {
                CompileError::raise(
                    &(*expr).base.base.location,
                    format_args!("Exceeded constant limit; simplify the code to compile"),
                );
            }

            (*self.bytecode).emit_abc(
                LuauOpcode::LOP_GETGLOBAL,
                target,
                0,
                bytecode_builder_get_string_hash(name) as u8,
            );
            (*self.bytecode).emit_aux(cid as u32);
        }
    }
}
