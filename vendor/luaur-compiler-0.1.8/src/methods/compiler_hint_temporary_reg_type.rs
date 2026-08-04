use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;

impl Compiler {
    pub fn hint_temporary_reg_type(
        &mut self,
        expr: *mut AstExpr,
        reg: i32,
        expected_type: LuauBytecodeType,
        inst_length: i32,
    ) {
        unsafe {
            if let Some(ty) = self.expr_types.find(&expr) {
                if *ty != expected_type {
                    let debug_pc = (*self.bytecode).get_debug_pc();
                    (*self.bytecode).push_local_type_info(
                        *ty,
                        reg as u8,
                        debug_pc - inst_length as u32,
                        debug_pc,
                    );
                }
            }
        }
    }
}
