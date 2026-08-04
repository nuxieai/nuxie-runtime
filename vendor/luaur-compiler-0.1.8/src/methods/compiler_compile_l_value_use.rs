use crate::enums::kind::Kind;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use crate::records::l_value::LValue;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_index_expr::AstExprIndexExpr;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_ast::records::ast_node::AstNode;
use luaur_bytecode::methods::bytecode_builder_get_string_hash::bytecode_builder_get_string_hash;
use luaur_common::enums::luau_bytecode_type::LBC_TYPE_TABLE;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_l_value_use(
        &mut self,
        lv: &LValue,
        reg: u8,
        set: bool,
        target_expr: *mut AstExpr,
    ) {
        // C++ `compileLValueUse` opens with `setDebugLine(lv.location)` so the store/load
        // instruction is attributed to the index's own location (e.g. the `["d"]` line of
        // `a["b"]["c"]["d"] = 4`), not whatever line was set while compiling the lvalue base.
        self.set_debug_line_location(&lv.location);
        unsafe {
            match lv.kind {
                Kind::Kind_Local => {
                    if set {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_MOVE, lv.reg, reg, 0);
                    } else {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_MOVE, reg, lv.reg, 0);
                    }
                }
                Kind::Kind_Upvalue => {
                    if set {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_SETUPVAL, reg, lv.upval, 0);
                    } else {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_GETUPVAL, reg, lv.upval, 0);
                    }
                }
                Kind::Kind_Global => {
                    let cid = (*self.bytecode).add_constant_string(lv.name);
                    if cid < 0 {
                        CompileError::raise(
                            &lv.location,
                            format_args!("Exceeded constant limit; simplify the code to compile"),
                        );
                    }

                    let hash = bytecode_builder_get_string_hash(lv.name) as u8;
                    if set {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_SETGLOBAL, reg, 0, hash);
                    } else {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_GETGLOBAL, reg, 0, hash);
                    }
                    (*self.bytecode).emit_aux(cid as u32);
                }
                Kind::Kind_IndexName => {
                    let cid = (*self.bytecode).add_constant_string(lv.name);
                    if cid < 0 {
                        CompileError::raise(
                            &lv.location,
                            format_args!("Exceeded constant limit; simplify the code to compile"),
                        );
                    }

                    let hash = bytecode_builder_get_string_hash(lv.name) as u8;
                    if set {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_SETTABLEKS, reg, lv.reg, hash);
                    } else {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_GETTABLEKS, reg, lv.reg, hash);
                    }
                    (*self.bytecode).emit_aux(cid as u32);

                    if !target_expr.is_null() {
                        let target_expr_index_name = luaur_ast::rtti::ast_node_as::<AstExprIndexName>(
                            target_expr as *mut AstNode,
                        );
                        if !target_expr_index_name.is_null() {
                            self.hint_temporary_expr_reg_type(
                                (*target_expr_index_name).expr,
                                lv.reg as i32,
                                LBC_TYPE_TABLE,
                                2,
                            );
                        }
                    }
                }
                Kind::Kind_IndexNumber => {
                    if set {
                        (*self.bytecode).emit_abc(
                            LuauOpcode::LOP_SETTABLEN,
                            reg,
                            lv.reg,
                            lv.number,
                        );
                    } else {
                        (*self.bytecode).emit_abc(
                            LuauOpcode::LOP_GETTABLEN,
                            reg,
                            lv.reg,
                            lv.number,
                        );
                    }

                    if !target_expr.is_null() {
                        let target_expr_index_expr = luaur_ast::rtti::ast_node_as::<AstExprIndexExpr>(
                            target_expr as *mut AstNode,
                        );
                        if !target_expr_index_expr.is_null() {
                            self.hint_temporary_expr_reg_type(
                                (*target_expr_index_expr).expr,
                                lv.reg as i32,
                                LBC_TYPE_TABLE,
                                1,
                            );
                        }
                    }
                }
                Kind::Kind_IndexExpr => {
                    if set {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_SETTABLE, reg, lv.reg, lv.index);
                    } else {
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_GETTABLE, reg, lv.reg, lv.index);
                    }

                    if !target_expr.is_null() {
                        let target_expr_index_expr = luaur_ast::rtti::ast_node_as::<AstExprIndexExpr>(
                            target_expr as *mut AstNode,
                        );
                        if !target_expr_index_expr.is_null() {
                            self.hint_temporary_expr_reg_type(
                                (*target_expr_index_expr).expr,
                                lv.reg as i32,
                                LBC_TYPE_TABLE,
                                1,
                            );
                            self.hint_temporary_expr_reg_type(
                                (*target_expr_index_expr).index,
                                lv.index as i32,
                                luaur_common::enums::luau_bytecode_type::LBC_TYPE_NUMBER,
                                1,
                            );
                        }
                    }
                }
                _ => LUAU_ASSERT!(false),
            }
        }
    }
}
