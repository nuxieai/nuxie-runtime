use crate::enums::type_constant_folding::Type;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr_binary::AstExprBinary;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_expr_and_or(&mut self, expr: *mut AstExprBinary, target: u8, target_temp: bool) {
        unsafe {
            let expr_ref = &*expr;
            let and_ = expr_ref.op == luaur_ast::records::ast_expr_binary::AstExprBinaryOp::And;
            let mut rs = self.reg_scope_compiler();

            if let Some(cl) = self.constants.find(&expr_ref.left) {
                if cl.r#type != Type::Type_Unknown {
                    self.compile_expr(
                        if and_ == cl.is_truthful() {
                            expr_ref.right
                        } else {
                            expr_ref.left
                        },
                        target,
                        target_temp,
                    );
                    return;
                }
            }

            if !self.is_condition_fast(expr_ref.left) {
                if let reg = self.get_expr_local_reg(expr_ref.right) {
                    if reg >= 0 {
                        let lr = self.compile_expr_auto(expr_ref.left, &mut rs);
                        (*self.bytecode).emit_abc(
                            if and_ {
                                LuauOpcode::LOP_AND
                            } else {
                                LuauOpcode::LOP_OR
                            },
                            target,
                            lr,
                            reg as u8,
                        );
                        return;
                    }
                }

                let cid = self.get_constant_index(expr_ref.right);
                if cid >= 0 && cid <= 255 {
                    let lr = self.compile_expr_auto(expr_ref.left, &mut rs);
                    (*self.bytecode).emit_abc(
                        if and_ {
                            LuauOpcode::LOP_ANDK
                        } else {
                            LuauOpcode::LOP_ORK
                        },
                        target,
                        lr,
                        cid as u8,
                    );
                    return;
                }
            }

            let reg = if target_temp {
                target
            } else {
                self.alloc_reg(expr as *mut _, 1)
            };
            let mut skip_jump = Vec::new();
            self.compile_condition_value(expr_ref.left, &reg, &mut skip_jump, !and_);
            self.compile_expr_temp(expr_ref.right, reg);
            let move_label = (*self.bytecode).emit_label();
            self.patch_jumps(expr as *mut _, &mut skip_jump, move_label);

            if target != reg {
                (*self.bytecode).emit_abc(LuauOpcode::LOP_MOVE, target, reg, 0);
            }
        }
    }
}
