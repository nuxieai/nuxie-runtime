use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::ast_stat_assign::AstStatAssign;
use luaur_ast::records::ast_stat_block::AstStatBlock;
use luaur_ast::records::ast_stat_break::AstStatBreak;
use luaur_ast::records::ast_stat_class::AstStatClass;
use luaur_ast::records::ast_stat_compound_assign::AstStatCompoundAssign;
use luaur_ast::records::ast_stat_continue::AstStatContinue;
use luaur_ast::records::ast_stat_for::AstStatFor;
use luaur_ast::records::ast_stat_for_in::AstStatForIn;
use luaur_ast::records::ast_stat_function::AstStatFunction;
use luaur_ast::records::ast_stat_if::AstStatIf;
use luaur_ast::records::ast_stat_local::AstStatLocal;
use luaur_ast::records::ast_stat_local_function::AstStatLocalFunction;
use luaur_ast::records::ast_stat_repeat::AstStatRepeat;
use luaur_ast::records::ast_stat_return::AstStatReturn;
use luaur_ast::records::ast_stat_while::AstStatWhile;
use luaur_ast::rtti;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_stat(&mut self, node: *mut AstStat) {
        unsafe {
            self.set_debug_line_ast_node(node as *mut AstNode);
            if self.options.coverage_level >= 1 && self.needs_coverage(node as *mut AstNode) {
                (*self.bytecode).emit_abc(LuauOpcode::LOP_COVERAGE, 0, 0, 0);
            }

            let stat = rtti::ast_node_as::<AstStatBlock>(node as *mut AstNode);
            if !stat.is_null() {
                let _rs = self.reg_scope_compiler();
                let old_locals = self.local_stack.len();
                if luaur_common::FFlag::LuauExportValueSyntax.get() {
                    self.block_depth += 1;
                }
                for i in 0..(*stat).body.size {
                    let body_stat = *(*stat).body.data.add(i);
                    self.compile_stat(body_stat);
                    if self.always_terminates(body_stat) {
                        break;
                    }
                }
                if luaur_common::FFlag::LuauExportValueSyntax.get() {
                    self.block_depth -= 1;
                }
                self.close_locals(old_locals);
                self.pop_locals(old_locals);
                return;
            }

            let stat = rtti::ast_node_as::<AstStatIf>(node as *mut AstNode);
            if !stat.is_null() {
                self.compile_stat_if(stat);
                return;
            }

            let stat = rtti::ast_node_as::<AstStatWhile>(node as *mut AstNode);
            if !stat.is_null() {
                self.compile_stat_while(stat);
                return;
            }

            let stat = rtti::ast_node_as::<AstStatRepeat>(node as *mut AstNode);
            if !stat.is_null() {
                self.compile_stat_repeat(stat);
                return;
            }

            if rtti::ast_node_is::<AstStatBreak>(&*(node as *mut AstNode)) {
                LUAU_ASSERT!(!self.loops.is_empty());
                self.close_locals(self.loops.last().unwrap().local_offset);
                let label = (*self.bytecode).emit_label();
                (*self.bytecode).emit_ad(LuauOpcode::LOP_JUMP, 0, 0);
                self.loop_jumps.push(crate::records::loop_jump::LoopJump {
                    r#type: crate::enums::type_compiler::Type::Break,
                    label,
                });
                return;
            }

            let stat = rtti::ast_node_as::<AstStatContinue>(node as *mut AstNode);
            if !stat.is_null() {
                LUAU_ASSERT!(!self.loops.is_empty());
                if self.loops.last().unwrap().continue_used.is_null() {
                    self.loops.last_mut().unwrap().continue_used = stat as *mut _;
                }
                self.close_locals(self.loops.last().unwrap().local_offset_continue);
                let label = (*self.bytecode).emit_label();
                (*self.bytecode).emit_ad(LuauOpcode::LOP_JUMP, 0, 0);
                self.loop_jumps.push(crate::records::loop_jump::LoopJump {
                    r#type: crate::enums::type_compiler::Type::Continue,
                    label,
                });
                return;
            }

            let stat = rtti::ast_node_as::<AstStatReturn>(node as *mut AstNode);
            if !stat.is_null() {
                if self.options.optimization_level >= 2 && !self.inline_frames.is_empty() {
                    self.compile_inline_return(stat, false);
                } else {
                    self.compile_stat_return(stat);
                }
                return;
            }

            let stat = rtti::ast_node_as::<luaur_ast::records::ast_stat_expr::AstStatExpr>(
                node as *mut AstNode,
            );
            if !stat.is_null() {
                let expr = rtti::ast_node_as::<luaur_ast::records::ast_expr_call::AstExprCall>(
                    (*stat).expr as *mut AstNode,
                );
                if !expr.is_null() {
                    self.compile_expr_call(expr, self.reg_top as u8, 0, false, false);
                } else {
                    self.compile_expr_side((*stat).expr);
                }
                return;
            }

            let stat = rtti::ast_node_as::<AstStatLocal>(node as *mut AstNode);
            if !stat.is_null() {
                if luaur_common::FFlag::LuauExportValueSyntax.get() {
                    for i in 0..(*stat).vars.size {
                        self.check_exported_local(
                            *(*stat).vars.data.add(i),
                            &(*stat).base.base.location,
                        );
                    }
                }
                self.compile_stat_local(stat);
                return;
            }

            let stat = rtti::ast_node_as::<AstStatFor>(node as *mut AstNode);
            if !stat.is_null() {
                self.compile_stat_for(stat);
                return;
            }

            let stat = rtti::ast_node_as::<AstStatForIn>(node as *mut AstNode);
            if !stat.is_null() {
                self.compile_stat_for_in(stat);
                return;
            }

            let stat = rtti::ast_node_as::<AstStatAssign>(node as *mut AstNode);
            if !stat.is_null() {
                self.compile_stat_assign(stat);
                return;
            }

            let stat = rtti::ast_node_as::<AstStatCompoundAssign>(node as *mut AstNode);
            if !stat.is_null() {
                self.compile_stat_compound_assign(stat);
                return;
            }

            let stat = rtti::ast_node_as::<AstStatFunction>(node as *mut AstNode);
            if !stat.is_null() {
                self.compile_stat_function(stat);
                return;
            }

            let stat = rtti::ast_node_as::<AstStatLocalFunction>(node as *mut AstNode);
            if !stat.is_null() {
                if luaur_common::FFlag::LuauExportValueSyntax.get() && (*(*stat).name).is_exported {
                    self.check_exported_local((*stat).name, &(*stat).base.base.location);

                    self.ensure_export_table(stat as *mut AstNode);

                    let _rs = self.reg_scope_compiler();
                    let var = self.alloc_reg(stat as *mut AstNode, 1);
                    self.compile_expr_function((*stat).func, var);

                    let name_ref =
                        crate::functions::sref_compiler::sref_ast_name((*(*stat).name).name);
                    let cid = (*self.bytecode).add_constant_string(name_ref);
                    if cid < 0 {
                        crate::records::compile_error::CompileError::raise(
                            &(*(*stat).name).location,
                            format_args!("Exceeded constant limit; simplify the code to compile"),
                        );
                    }

                    let table_reg = self.get_export_table_reg(stat as *mut AstNode);
                    (*self.bytecode).emit_abc(
                        LuauOpcode::LOP_SETTABLEKS,
                        var,
                        table_reg,
                        luaur_bytecode::methods::bytecode_builder_get_string_hash::bytecode_builder_get_string_hash(name_ref) as u8,
                    );
                    (*self.bytecode).emit_aux(cid as u32);
                } else {
                    // kDefaultAllocPc sentinel (push_local treats !0 as "use debugpc").
                    let var = self.alloc_reg(stat as *mut AstNode, 1);

                    self.push_local((*stat).name, var, !0u32);
                    if luaur_common::FFlag::LuauExportValueSyntax.get() {
                        self.check_exported_local((*stat).name, &(*stat).base.base.location);
                    }
                    self.compile_expr_function((*stat).func, var);

                    // We *have* to pushLocal before compiling the function (it may refer
                    // to the local as an upvalue), but that means the local's debugpc is
                    // an instruction before its value is established; fix it up afterwards.
                    let debugpc = (*self.bytecode).get_debug_pc();
                    self.locals.get_or_insert((*stat).name).debugpc = debugpc;
                }
                return;
            }

            if luaur_common::FFlag::DebugLuauUserDefinedClasses.get() {
                let stat = rtti::ast_node_as::<AstStatClass>(node as *mut AstNode);
                if !stat.is_null() {
                    self.compile_class_declaration(stat);
                }
            }
        }
    }
}
