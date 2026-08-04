use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr_binary::AstExprBinaryOp;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn get_jump_op_compare(&mut self, op: AstExprBinaryOp, not_: bool) -> LuauOpcode {
        match op {
            AstExprBinaryOp::CompareNe => {
                if not_ {
                    LuauOpcode::LOP_JUMPIFEQ
                } else {
                    LuauOpcode::LOP_JUMPIFNOTEQ
                }
            }
            AstExprBinaryOp::CompareEq => {
                if not_ {
                    LuauOpcode::LOP_JUMPIFNOTEQ
                } else {
                    LuauOpcode::LOP_JUMPIFEQ
                }
            }
            AstExprBinaryOp::CompareLt | AstExprBinaryOp::CompareGt => {
                if not_ {
                    LuauOpcode::LOP_JUMPIFNOTLT
                } else {
                    LuauOpcode::LOP_JUMPIFLT
                }
            }
            AstExprBinaryOp::CompareLe | AstExprBinaryOp::CompareGe => {
                if not_ {
                    LuauOpcode::LOP_JUMPIFNOTLE
                } else {
                    LuauOpcode::LOP_JUMPIFLE
                }
            }
            _ => {
                LUAU_ASSERT!(false);
                LuauOpcode::LOP_NOP
            }
        }
    }
}
