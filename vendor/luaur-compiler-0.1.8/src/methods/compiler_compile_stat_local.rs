use crate::functions::sref_compiler::sref_ast_name;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat_local::AstStatLocal;
use luaur_bytecode::methods::bytecode_builder_get_string_hash::bytecode_builder_get_string_hash;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_stat_local(&mut self, stat: *mut AstStatLocal) {
        unsafe {
            if self.options.optimization_level >= 1
                && self.options.debug_level <= 1
                && self.are_locals_redundant(stat)
            {
                return;
            }

            let stat_ref = &*stat;

            if self.options.optimization_level >= 1
                && stat_ref.vars.size == 1
                && stat_ref.values.size == 1
            {
                let value = *stat_ref.values.data;
                let re = self.get_expr_local(value);

                if !re.is_null() {
                    let local = *stat_ref.vars.data;
                    let lv_written = self.variables.find(&local).map_or(false, |lv| lv.written);
                    let rv_local = (*re).local;
                    let rv_written = self
                        .variables
                        .find(&rv_local)
                        .map_or(false, |rv| rv.written);
                    let reg = self.get_expr_local_reg(value);

                    if reg >= 0
                        && !lv_written
                        && !rv_written
                        && !(*local).is_exported
                        && !(*rv_local).is_exported
                    {
                        let allocpc = (*self.bytecode).get_debug_pc();
                        self.push_local(local, reg as u8, allocpc);
                        return;
                    }
                }
            }

            let vars = self.alloc_reg(stat as *mut AstNode, stat_ref.vars.size as u32);
            let allocpc = (*self.bytecode).get_debug_pc();

            self.compile_expr_list_temp(&stat_ref.values, vars, stat_ref.vars.size as u8, true);

            for i in 0..stat_ref.vars.size {
                let local = *stat_ref.vars.data.add(i);

                if luaur_common::FFlag::LuauExportValueSyntax.get() && (*local).is_exported {
                    self.ensure_export_table(stat as *mut AstNode);

                    let name_ref = sref_ast_name((*local).name);
                    let cid = (*self.bytecode).add_constant_string(name_ref);
                    if cid < 0 {
                        CompileError::raise(
                            &(*local).location,
                            format_args!("Exceeded constant limit; simplify the code to compile"),
                        );
                    }

                    let table_reg = self.get_export_table_reg(stat as *mut AstNode);
                    (*self.bytecode).emit_abc(
                        LuauOpcode::LOP_SETTABLEKS,
                        vars + i as u8,
                        table_reg,
                        bytecode_builder_get_string_hash(name_ref) as u8,
                    );
                    (*self.bytecode).emit_aux(cid as u32);
                } else {
                    self.push_local(local, vars + i as u8, allocpc);
                }
            }
        }
    }
}
