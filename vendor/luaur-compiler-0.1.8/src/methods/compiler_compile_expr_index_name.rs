use crate::functions::sref_compiler::sref_ast_name;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr_global::AstExprGlobal;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_bytecode::methods::bytecode_builder_get_string_hash::bytecode_builder_get_string_hash;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_expr_index_name(
        &mut self,
        expr: *mut AstExprIndexName,
        target: u8,
        target_temp: bool,
    ) {
        unsafe {
            let expr_ref = &*expr;
            self.set_debug_line_ast_node(expr as *mut _);

            let mut import_root: *mut AstExprGlobal = core::ptr::null_mut();
            let mut import1: *mut AstExprIndexName = core::ptr::null_mut();
            let mut import2: *mut AstExprIndexName = core::ptr::null_mut();

            let index = luaur_ast::rtti::ast_node_as::<AstExprIndexName>(
                expr_ref.expr as *mut luaur_ast::records::ast_node::AstNode,
            );
            if !index.is_null() {
                import_root = luaur_ast::rtti::ast_node_as::<AstExprGlobal>(
                    (*index).expr as *mut luaur_ast::records::ast_node::AstNode,
                );
                import1 = index;
                import2 = expr;
            } else {
                import_root = luaur_ast::rtti::ast_node_as::<AstExprGlobal>(
                    expr_ref.expr as *mut luaur_ast::records::ast_node::AstNode,
                );
                import1 = expr;
            }

            if !import_root.is_null() && self.can_import_chain(import_root) {
                let id0 = (*self.bytecode).add_constant_string(sref_ast_name((*import_root).name));
                let id1 = (*self.bytecode).add_constant_string(sref_ast_name((*import1).index));
                let id2 = if !import2.is_null() {
                    (*self.bytecode).add_constant_string(sref_ast_name((*import2).index))
                } else {
                    -1
                };

                if id0 >= 0 && id1 >= 0 && (import2.is_null() || id2 >= 0) {
                    if id0 < 1024 && id1 < 1024 && (import2.is_null() || id2 < 1024) {
                        let iid = if !import2.is_null() {
                            get_import_id_3(id0, id1, id2)
                        } else {
                            get_import_id_2(id0, id1)
                        };
                        let cid = (*self.bytecode).add_import(iid);
                        if cid >= 0 && cid < 32768 {
                            (*self.bytecode).emit_ad(LuauOpcode::LOP_GETIMPORT, target, cid as i16);
                            (*self.bytecode).emit_aux(iid as u32);
                            return;
                        }
                    }
                }
            }

            let mut rs = self.reg_scope_compiler();
            let reg = if let local_reg = self.get_expr_local_reg(expr_ref.expr) {
                if local_reg >= 0 {
                    local_reg as u8
                } else if target_temp {
                    self.compile_expr_temp(expr_ref.expr, target);
                    target
                } else {
                    self.compile_expr_auto(expr_ref.expr, &mut rs)
                }
            } else {
                self.compile_expr_auto(expr_ref.expr, &mut rs)
            };

            self.set_debug_line_location(&expr_ref.index_location);
            let iname = sref_ast_name(expr_ref.index);
            let cid = (*self.bytecode).add_constant_string(iname);
            if cid < 0 {
                CompileError::raise(
                    &expr_ref.base.base.location,
                    format_args!("Exceeded constant limit; simplify the code to compile"),
                );
            }

            (*self.bytecode).emit_abc(
                LuauOpcode::LOP_GETTABLEKS,
                target,
                reg,
                bytecode_builder_get_string_hash(iname) as u8,
            );
            (*self.bytecode).emit_aux(cid as u32);
            self.hint_temporary_expr_reg_type(expr_ref.expr, reg as i32, LuauBytecodeType(4), 2);
        }
    }
}

fn get_import_id_2(id0: i32, id1: i32) -> u32 {
    ((2u32) << 30) | ((id0 as u32) << 20) | ((id1 as u32) << 10)
}

fn get_import_id_3(id0: i32, id1: i32, id2: i32) -> u32 {
    ((3u32) << 30) | ((id0 as u32) << 20) | ((id1 as u32) << 10) | id2 as u32
}
