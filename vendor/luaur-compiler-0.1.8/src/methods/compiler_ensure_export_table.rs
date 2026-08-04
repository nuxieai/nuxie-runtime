use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

const K_DEFAULT_ALLOC_PC: u32 = !0u32;

impl Compiler {
    pub fn ensure_export_table(&mut self, node: *mut AstNode) {
        let export_local = &mut self.export_table_local as *mut _;
        if self.locals.contains(&export_local) {
            return;
        }

        LUAU_ASSERT!(self.at_top_level());

        let table_reg = self.alloc_reg(node, 1);
        unsafe {
            (*self.bytecode).emit_abc(
                LuauOpcode::LOP_NEWTABLE,
                table_reg,
                Compiler::encode_hash_size(0),
                0,
            );
            (*self.bytecode).emit_aux(0);
        }

        self.push_local(export_local, table_reg, K_DEFAULT_ALLOC_PC);
    }
}
