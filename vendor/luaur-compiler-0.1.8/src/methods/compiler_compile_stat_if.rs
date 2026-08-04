use crate::enums::type_compiler::Type;
use crate::records::compiler::Compiler;
use crate::records::loop_jump::LoopJump;
use luaur_ast::records::ast_expr_binary::AstExprBinary;
use luaur_ast::records::ast_expr_binary::AstExprBinary_Op;
use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::records::ast_stat_block::AstStatBlock;
use luaur_ast::records::ast_stat_continue::AstStatContinue;
use luaur_ast::records::ast_stat_if::AstStatIf;
use luaur_ast::rtti::ast_node_as;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_stat_if(&mut self, stat: *mut AstStatIf) {
        unsafe {
            let stat = &*stat;

            if self.is_constant_false(stat.condition) {
                if !stat.elsebody.is_null() {
                    self.compile_stat(stat.elsebody);
                }
                return;
            }

            let cand = ast_node_as::<AstExprBinary>(
                stat.condition as *mut luaur_ast::records::ast_node::AstNode,
            );
            if !cand.is_null() {
                let cand = &*cand;
                if cand.op == AstExprBinary_Op::And && self.is_constant_false(cand.right) {
                    self.compile_expr_side(cand.left);
                    if !stat.elsebody.is_null() {
                        self.compile_stat(stat.elsebody);
                    }
                    return;
                }
            }

            if stat.elsebody.is_null()
                && self.is_stat_break(stat.thenbody as *mut AstStat)
                && !self.are_locals_captured(self.loops.last().unwrap().local_offset)
            {
                let mut else_jump = Vec::new();
                self.compile_condition_value(
                    stat.condition,
                    core::ptr::null(),
                    &mut else_jump,
                    true,
                );
                for jump in else_jump {
                    self.loop_jumps.push(LoopJump {
                        r#type: Type::Break,
                        label: jump,
                    });
                }
                return;
            }

            let continue_statement = self.extract_stat_continue(stat.thenbody as *mut AstStatBlock);
            if stat.elsebody.is_null()
                && !continue_statement.is_null()
                && !self.are_locals_captured(self.loops.last().unwrap().local_offset_continue)
            {
                if self.loops.last().unwrap().continue_used.is_null() {
                    self.loops.last_mut().unwrap().continue_used = continue_statement;
                }
                let mut else_jump = Vec::new();
                self.compile_condition_value(
                    stat.condition,
                    core::ptr::null(),
                    &mut else_jump,
                    true,
                );
                for jump in else_jump {
                    self.loop_jumps.push(LoopJump {
                        r#type: Type::Continue,
                        label: jump,
                    });
                }
                return;
            }

            let mut else_jump = Vec::new();
            self.compile_condition_value(stat.condition, core::ptr::null(), &mut else_jump, false);
            self.compile_stat(stat.thenbody as *mut AstStat);

            if !stat.elsebody.is_null() && !else_jump.is_empty() {
                if self.always_terminates(stat.thenbody as *mut AstStat) {
                    let else_label = unsafe { (*self.bytecode).emit_label() };
                    self.compile_stat(stat.elsebody);
                    self.patch_jumps(
                        stat as *const AstStatIf as *mut luaur_ast::records::ast_node::AstNode,
                        &mut else_jump,
                        else_label,
                    );
                } else {
                    let then_label = unsafe { (*self.bytecode).emit_label() };
                    unsafe { (*self.bytecode).emit_ad(LuauOpcode::LOP_JUMP, 0, 0) };
                    let else_label = unsafe { (*self.bytecode).emit_label() };
                    self.compile_stat(stat.elsebody);
                    let end_label = unsafe { (*self.bytecode).emit_label() };
                    self.patch_jumps(
                        stat as *const AstStatIf as *mut luaur_ast::records::ast_node::AstNode,
                        &mut else_jump,
                        else_label,
                    );
                    self.patch_jump(
                        stat as *const AstStatIf as *mut luaur_ast::records::ast_node::AstNode,
                        then_label,
                        end_label,
                    );
                }
            } else {
                let end_label = unsafe { (*self.bytecode).emit_label() };
                self.patch_jumps(
                    stat as *const AstStatIf as *mut luaur_ast::records::ast_node::AstNode,
                    &mut else_jump,
                    end_label,
                );
            }
        }
    }
}
