use crate::enums::type_constant_folding::Type;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr_binary::{AstExprBinary, AstExprBinaryOp};
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_expr_binary(
        &mut self,
        expr: *mut AstExprBinary,
        target: u8,
        _target_temp: bool,
    ) {
        unsafe {
            let expr_ref = &*expr;
            let mut rs = self.reg_scope_compiler();

            match expr_ref.op {
                AstExprBinaryOp::Add
                | AstExprBinaryOp::Sub
                | AstExprBinaryOp::Mul
                | AstExprBinaryOp::Div
                | AstExprBinaryOp::FloorDiv
                | AstExprBinaryOp::Mod
                | AstExprBinaryOp::Pow => {
                    let rc = self.get_constant_number(expr_ref.right);
                    if rc >= 0 && rc <= 255 {
                        let rl = self.compile_expr_auto(expr_ref.left, &mut rs);
                        (*self.bytecode).emit_abc(
                            self.get_binary_op_arith(expr_ref.op, true),
                            target,
                            rl,
                            rc as u8,
                        );
                        self.hint_temporary_expr_reg_type(
                            expr_ref.left,
                            rl as i32,
                            LuauBytecodeType(2),
                            1,
                        );
                    } else {
                        if expr_ref.op == AstExprBinaryOp::Sub
                            || expr_ref.op == AstExprBinaryOp::Div
                        {
                            let lc = self.get_constant_number(expr_ref.left);
                            if lc >= 0 && lc <= 255 {
                                let rr = self.compile_expr_auto(expr_ref.right, &mut rs);
                                let op = if expr_ref.op == AstExprBinaryOp::Sub {
                                    LuauOpcode::LOP_SUBRK
                                } else {
                                    LuauOpcode::LOP_DIVRK
                                };
                                (*self.bytecode).emit_abc(op, target, lc as u8, rr);
                                self.hint_temporary_expr_reg_type(
                                    expr_ref.right,
                                    rr as i32,
                                    LuauBytecodeType(2),
                                    1,
                                );
                                return;
                            }
                        } else if self.options.optimization_level >= 2
                            && (expr_ref.op == AstExprBinaryOp::Add
                                || expr_ref.op == AstExprBinaryOp::Mul)
                        {
                            // Optimization: replace k*r with r*k when r is known to be a number (otherwise
                            // metamethods may be called). For vectors this only makes sense for multiplication
                            // since number+vector is an error.
                            if let Some(ty) = self
                                .expr_types
                                .find(&(expr as *mut luaur_ast::records::ast_expr::AstExpr))
                                .copied()
                            {
                                if ty == LuauBytecodeType(2)
                                    || (ty == LuauBytecodeType(8)
                                        && expr_ref.op == AstExprBinaryOp::Mul)
                                {
                                    let lc = self.get_constant_number(expr_ref.left);
                                    if lc >= 0 && lc <= 255 {
                                        let rr = self.compile_expr_auto(expr_ref.right, &mut rs);
                                        (*self.bytecode).emit_abc(
                                            self.get_binary_op_arith(expr_ref.op, true),
                                            target,
                                            rr,
                                            lc as u8,
                                        );
                                        self.hint_temporary_expr_reg_type(
                                            expr_ref.right,
                                            rr as i32,
                                            LuauBytecodeType(2),
                                            1,
                                        );
                                        return;
                                    }
                                }
                            }
                        }
                        let rl = self.compile_expr_auto(expr_ref.left, &mut rs);
                        let rr = self.compile_expr_auto(expr_ref.right, &mut rs);
                        (*self.bytecode).emit_abc(
                            self.get_binary_op_arith(expr_ref.op, false),
                            target,
                            rl,
                            rr,
                        );
                        self.hint_temporary_expr_reg_type(
                            expr_ref.left,
                            rl as i32,
                            LuauBytecodeType(2),
                            1,
                        );
                        self.hint_temporary_expr_reg_type(
                            expr_ref.right,
                            rr as i32,
                            LuauBytecodeType(2),
                            1,
                        );
                    }
                }
                AstExprBinaryOp::Concat => {
                    let mut args = vec![expr_ref.left, expr_ref.right];
                    self.unroll_concats(&mut args);
                    let regs = self.alloc_reg(expr as *mut _, args.len() as u32);
                    for (i, &arg) in args.iter().enumerate() {
                        self.compile_expr_temp(arg, regs + i as u8);
                    }
                    (*self.bytecode).emit_abc(
                        LuauOpcode::LOP_CONCAT,
                        target,
                        regs,
                        regs + args.len() as u8 - 1,
                    );
                }
                AstExprBinaryOp::CompareNe
                | AstExprBinaryOp::CompareEq
                | AstExprBinaryOp::CompareLt
                | AstExprBinaryOp::CompareLe
                | AstExprBinaryOp::CompareGt
                | AstExprBinaryOp::CompareGe => {
                    let jump_label = self.compile_compare_jump(expr, false);
                    (*self.bytecode).emit_abc(LuauOpcode::LOP_LOADB, target, 0, 1);
                    let then_label = (*self.bytecode).emit_label();
                    (*self.bytecode).emit_abc(LuauOpcode::LOP_LOADB, target, 1, 0);
                    self.patch_jump(expr as *mut _, jump_label, then_label);
                }
                AstExprBinaryOp::And | AstExprBinaryOp::Or => {
                    self.compile_expr_and_or(expr, target, _target_temp);
                }
                _ => LUAU_ASSERT!(false),
            }
        }
    }
}
