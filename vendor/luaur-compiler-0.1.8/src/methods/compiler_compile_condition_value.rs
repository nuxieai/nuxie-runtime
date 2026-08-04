use crate::enums::type_constant_folding::Type;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_binary::{AstExprBinary, AstExprBinaryOp};
use luaur_ast::records::ast_expr_group::AstExprGroup;
use luaur_ast::records::ast_expr_unary::{AstExprUnary, AstExprUnaryOp};
use luaur_ast::rtti;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_condition_value(
        &mut self,
        node: *mut AstExpr,
        target: *const u8,
        skip_jump: &mut Vec<usize>,
        only_truth: bool,
    ) {
        if let Some(cv) = self.constants.find(&node) {
            if cv.r#type != Type::Type_Unknown {
                if cv.is_truthful() == only_truth {
                    if !target.is_null() {
                        self.compile_expr_temp(node, unsafe { *target });
                    }
                    skip_jump.push(unsafe { (*self.bytecode).emit_label() });
                    unsafe {
                        (*self.bytecode).emit_ad(LuauOpcode::LOP_JUMP, 0, 0);
                    }
                }
                return;
            }
        }

        let expr = unsafe { rtti::ast_node_as::<AstExprBinary>(node as *mut _) };
        if !expr.is_null() {
            match unsafe { (*expr).op } {
                AstExprBinaryOp::And | AstExprBinaryOp::Or => {
                    if only_truth == (unsafe { (*expr).op } == AstExprBinaryOp::And) {
                        let mut else_jump = Vec::new();
                        self.compile_condition_value(
                            unsafe { (*expr).left },
                            core::ptr::null(),
                            &mut else_jump,
                            !only_truth,
                        );
                        self.compile_condition_value(
                            unsafe { (*expr).right },
                            target,
                            skip_jump,
                            only_truth,
                        );
                        let else_label = unsafe { (*self.bytecode).emit_label() };
                        self.patch_jumps(node as *mut _, &mut else_jump, else_label);
                    } else {
                        self.compile_condition_value(
                            unsafe { (*expr).left },
                            target,
                            skip_jump,
                            only_truth,
                        );
                        self.compile_condition_value(
                            unsafe { (*expr).right },
                            target,
                            skip_jump,
                            only_truth,
                        );
                    }
                    return;
                }
                AstExprBinaryOp::CompareNe
                | AstExprBinaryOp::CompareEq
                | AstExprBinaryOp::CompareLt
                | AstExprBinaryOp::CompareLe
                | AstExprBinaryOp::CompareGt
                | AstExprBinaryOp::CompareGe => {
                    if !target.is_null() {
                        unsafe {
                            (*self.bytecode).emit_abc(
                                LuauOpcode::LOP_LOADB,
                                *target,
                                if only_truth { 1 } else { 0 },
                                0,
                            );
                        }
                    }
                    let jump_label = self.compile_compare_jump(expr, !only_truth);
                    skip_jump.push(jump_label);
                    return;
                }
                _ => {}
            }
        }

        let expr = unsafe { rtti::ast_node_as::<AstExprUnary>(node as *mut _) };
        if !expr.is_null() {
            if target.is_null() && unsafe { (*expr).op } == AstExprUnaryOp::Not {
                self.compile_condition_value(
                    unsafe { (*expr).expr },
                    target,
                    skip_jump,
                    !only_truth,
                );
                return;
            }
        }

        let expr = unsafe { rtti::ast_node_as::<AstExprGroup>(node as *mut _) };
        if !expr.is_null() {
            self.compile_condition_value(unsafe { (*expr).expr }, target, skip_jump, only_truth);
            return;
        }

        let mut rs = self.reg_scope_compiler();
        let reg = if !target.is_null() {
            unsafe {
                self.compile_expr_temp(node, *target);
                *target
            }
        } else {
            self.compile_expr_auto(node, &mut rs)
        };

        skip_jump.push(unsafe { (*self.bytecode).emit_label() });
        unsafe {
            (*self.bytecode).emit_ad(
                if only_truth {
                    LuauOpcode::LOP_JUMPIF
                } else {
                    LuauOpcode::LOP_JUMPIFNOT
                },
                reg,
                0,
            );
        }
    }
}
