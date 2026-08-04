use luaur_ast::records::ast_expr_unary::AstExprUnaryOp;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl crate::records::compiler::Compiler {
    pub fn get_unary_op(&mut self, op: AstExprUnaryOp) -> LuauOpcode {
        match op {
            AstExprUnaryOp::Not => LuauOpcode::LOP_NOT,
            AstExprUnaryOp::Minus => LuauOpcode::LOP_MINUS,
            AstExprUnaryOp::Len => LuauOpcode::LOP_LENGTH,
            _ => {
                LUAU_ASSERT!(false);
                LuauOpcode::LOP_NOP
            }
        }
    }
}
