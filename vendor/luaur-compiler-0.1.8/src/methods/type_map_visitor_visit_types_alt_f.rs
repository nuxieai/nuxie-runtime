use crate::functions::sref_compiler::sref_ast_name;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_bytecode::records::string_ref::StringRef;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_expr_global(&mut self, expr: *mut AstExprGlobal, target: u8) {
        unsafe {
            if self.can_import(expr) {
                let name = (*expr).name;
                let id0 = (*self.bytecode).add_constant_string(sref_ast_name(name));

                if id0 < 0 {
                    let location = (*expr).base.base.location;
                    CompileError::raise(
                        &location,
                        core::format_args!("Exceeded constant limit; simplify the code to compile"),
                    );
                }

                if id0 < 1024 {
                    let iid = BytecodeBuilder::get_import_id(id0);
                    let cid = (*self.bytecode).add_import(iid);

                    if cid >= 0 && cid < 32768 {
                        (*self.bytecode).emit_ad(LuauOpcode::LOP_GETIMPORT, target, cid as i16);
                        (*self.bytecode).emit_aux(iid);
                        return;
                    }
                }
            }

            let name = (*expr).name;
            let gname = sref_ast_name(name);
            let cid = (*self.bytecode).add_constant_string(gname);

            if cid < 0 {
                let location = (*expr).base.base.location;
                CompileError::raise(
                    &location,
                    core::format_args!("Exceeded constant limit; simplify the code to compile"),
                );
            }

            let hash = BytecodeBuilder::get_string_hash(gname) & 0xFF;
            (*self.bytecode).emit_abc(LuauOpcode::LOP_GETGLOBAL, target, 0, hash as u8);
            (*self.bytecode).emit_aux(cid as u32);
        }
    }
}
