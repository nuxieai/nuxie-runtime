use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_expr_if_else_and_or(
        &mut self,
        and_: bool,
        creg: u8,
        other: *mut AstExpr,
        target: u8,
    ) {
        let cid = self.get_constant_index(other);
        unsafe {
            if cid >= 0 && cid <= 255 {
                (*self.bytecode).emit_abc(
                    if and_ {
                        LuauOpcode::LOP_ANDK
                    } else {
                        LuauOpcode::LOP_ORK
                    },
                    target,
                    creg,
                    cid as u8,
                );
            } else {
                let mut rs = self.reg_scope_compiler();
                let oreg = self.compile_expr_auto(other, &mut rs);
                (*self.bytecode).emit_abc(
                    if and_ {
                        LuauOpcode::LOP_AND
                    } else {
                        LuauOpcode::LOP_OR
                    },
                    target,
                    creg,
                    oreg,
                );
            }
        }
    }
}
