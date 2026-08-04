use crate::enums::type_constant_folding::Type;
use crate::records::assignment::Assignment;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr_binary::AstExprBinaryOp;
use luaur_ast::records::ast_stat_compound_assign::AstStatCompoundAssign;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_stat_compound_assign(&mut self, stat: *mut AstStatCompoundAssign) {
        unsafe {
            let stat_ref = &*stat;
            let mut rs = self.reg_scope_compiler();
            let var = self.compile_l_value(stat_ref.var, &mut rs);
            let target = if var.kind == crate::enums::kind::Kind::Kind_Local {
                var.reg
            } else {
                self.alloc_reg(stat as *mut _, 1)
            };

            match stat_ref.op {
                AstExprBinaryOp::Add
                | AstExprBinaryOp::Sub
                | AstExprBinaryOp::Mul
                | AstExprBinaryOp::Div
                | AstExprBinaryOp::FloorDiv
                | AstExprBinaryOp::Mod
                | AstExprBinaryOp::Pow => {
                    if var.kind != crate::enums::kind::Kind::Kind_Local {
                        self.compile_l_value_use(&var, target, false, stat_ref.var);
                    }
                    let rc = self.get_constant_number(stat_ref.value);
                    if rc >= 0 && rc <= 255 {
                        (*self.bytecode).emit_abc(
                            self.get_binary_op_arith(stat_ref.op, true),
                            target,
                            target,
                            rc as u8,
                        );
                    } else {
                        let rr = self.compile_expr_auto(stat_ref.value, &mut rs);
                        (*self.bytecode).emit_abc(
                            self.get_binary_op_arith(stat_ref.op, false),
                            target,
                            target,
                            rr,
                        );
                        if var.kind != crate::enums::kind::Kind::Kind_Local {
                            self.hint_temporary_reg_type(
                                stat_ref.var,
                                target as i32,
                                LuauBytecodeType(2),
                                1,
                            );
                        }
                        self.hint_temporary_expr_reg_type(
                            stat_ref.value,
                            rr as i32,
                            LuauBytecodeType(2),
                            1,
                        );
                    }
                }
                AstExprBinaryOp::Concat => {
                    let mut args = vec![stat_ref.value];
                    self.unroll_concats(&mut args);
                    let regs = self.alloc_reg(stat as *mut _, (1 + args.len()) as u32);
                    self.compile_l_value_use(&var, regs, false, stat_ref.var);
                    for (i, &arg) in args.iter().enumerate() {
                        self.compile_expr_temp(arg, regs + 1 + i as u8);
                    }
                    (*self.bytecode).emit_abc(
                        LuauOpcode::LOP_CONCAT,
                        target,
                        regs,
                        regs + args.len() as u8,
                    );
                }
                _ => LUAU_ASSERT!(false),
            }

            if var.kind != crate::enums::kind::Kind::Kind_Local {
                self.compile_assign(&var, target, stat_ref.var);
            }
        }
    }
}
