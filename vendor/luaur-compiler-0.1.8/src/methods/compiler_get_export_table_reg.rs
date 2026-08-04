use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn get_export_table_reg(&mut self, node: *mut AstNode) -> u8 {
        let local_ptr = &mut self.export_table_local as *mut _;
        let reg = self.get_local_reg(local_ptr);
        if reg >= 0 {
            return reg as u8;
        }

        let upval = self.get_upval(local_ptr);
        let reg = self.alloc_reg(node, 1);
        unsafe {
            (*self.bytecode).emit_abc(LuauOpcode::LOP_GETUPVAL, reg, upval, 0);
        }
        reg
    }
}
