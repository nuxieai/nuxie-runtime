use crate::enums::type_constant_folding::Type;
use crate::functions::sref_compiler::sref_ast_name;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use luaur_ast::records::ast_expr_binary::{AstExprBinary, AstExprBinaryOp};
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_compare_jump(&mut self, expr: *mut AstExprBinary, not_: bool) -> usize {
        unsafe {
            let expr_ref = &*expr;
            let mut left = expr_ref.left;
            let mut right = expr_ref.right;
            let is_eq = expr_ref.op == AstExprBinaryOp::CompareEq
                || expr_ref.op == AstExprBinaryOp::CompareNe;

            let mut operand_is_constant = self.is_constant(right);
            if is_eq && !operand_is_constant {
                operand_is_constant = self.is_constant(left);
                if operand_is_constant {
                    core::mem::swap(&mut left, &mut right);
                }
            }

            if operand_is_constant
                && (self.is_constant_vector(right) || self.is_constant_integer(right))
            {
                operand_is_constant = false;
            }

            let mut rs = self.reg_scope_compiler();
            let rl = self.compile_expr_auto(left, &mut rs);

            if is_eq && operand_is_constant {
                let cv = self.get_constant(right);
                LUAU_ASSERT!(cv.r#type != Type::Type_Unknown);

                let (opc, cid_val) = match cv.r#type {
                    Type::Type_Nil => (LuauOpcode::LOP_JUMPXEQKNIL, 0),
                    Type::Type_Boolean => (LuauOpcode::LOP_JUMPXEQKB, cv.data.value_boolean as i32),
                    Type::Type_Number => {
                        (LuauOpcode::LOP_JUMPXEQKN, self.get_constant_index(right))
                    }
                    Type::Type_String => {
                        (LuauOpcode::LOP_JUMPXEQKS, self.get_constant_index(right))
                    }
                    _ => {
                        LUAU_ASSERT!(false);
                        (LuauOpcode::LOP_NOP, 0)
                    }
                };

                if cid_val < 0 {
                    CompileError::raise(
                        &expr_ref.base.base.location,
                        format_args!("Exceeded constant limit; simplify the code to compile"),
                    );
                }

                let jump_label = (*self.bytecode).emit_label();
                let flip = if (expr_ref.op == AstExprBinaryOp::CompareEq) == not_ {
                    0x80000000
                } else {
                    0
                };

                (*self.bytecode).emit_ad(opc, rl, 0);
                (*self.bytecode).emit_aux((cid_val as u32) | flip);

                jump_label
            } else {
                let opc = self.get_jump_op_compare(expr_ref.op, not_);
                let rr = self.compile_expr_auto(right, &mut rs);
                let jump_label = (*self.bytecode).emit_label();

                if expr_ref.op == AstExprBinaryOp::CompareGt
                    || expr_ref.op == AstExprBinaryOp::CompareGe
                {
                    (*self.bytecode).emit_ad(opc, rr, 0);
                    (*self.bytecode).emit_aux(rl as u32);
                } else {
                    (*self.bytecode).emit_ad(opc, rl, 0);
                    (*self.bytecode).emit_aux(rr as u32);
                }
                jump_label
            }
        }
    }
}
