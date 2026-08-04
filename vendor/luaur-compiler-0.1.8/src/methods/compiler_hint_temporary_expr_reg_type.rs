use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;

impl Compiler {
    pub fn hint_temporary_expr_reg_type(
        &mut self,
        expr: *mut AstExpr,
        reg: i32,
        expected_type: LuauBytecodeType,
        inst_length: i32,
    ) {
        // If we allocated a temporary register for the operation argument, try hinting its type
        if self.get_expr_local(expr).is_null() {
            self.hint_temporary_reg_type(expr, reg, expected_type, inst_length);
        }
    }
}
