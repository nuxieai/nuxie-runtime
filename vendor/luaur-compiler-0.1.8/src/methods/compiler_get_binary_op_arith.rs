use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_binary::AstExprBinaryOp;
use luaur_common::enums::luau_opcode::LuauOpcode;

use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn get_binary_op_arith(&mut self, op: AstExprBinaryOp, k: bool) -> LuauOpcode {
        match op {
            AstExprBinaryOp::Add => {
                if k {
                    LuauOpcode::LOP_ADDK
                } else {
                    LuauOpcode::LOP_ADD
                }
            }
            AstExprBinaryOp::Sub => {
                if k {
                    LuauOpcode::LOP_SUBK
                } else {
                    LuauOpcode::LOP_SUB
                }
            }
            AstExprBinaryOp::Mul => {
                if k {
                    LuauOpcode::LOP_MULK
                } else {
                    LuauOpcode::LOP_MUL
                }
            }
            AstExprBinaryOp::Div => {
                if k {
                    LuauOpcode::LOP_DIVK
                } else {
                    LuauOpcode::LOP_DIV
                }
            }
            AstExprBinaryOp::FloorDiv => {
                if k {
                    LuauOpcode::LOP_IDIVK
                } else {
                    LuauOpcode::LOP_IDIV
                }
            }
            AstExprBinaryOp::Mod => {
                if k {
                    LuauOpcode::LOP_MODK
                } else {
                    LuauOpcode::LOP_MOD
                }
            }
            AstExprBinaryOp::Pow => {
                if k {
                    LuauOpcode::LOP_POWK
                } else {
                    LuauOpcode::LOP_POW
                }
            }
            _ => {
                LUAU_ASSERT!(false);
                LuauOpcode::LOP_NOP
            }
        }
    }
}
