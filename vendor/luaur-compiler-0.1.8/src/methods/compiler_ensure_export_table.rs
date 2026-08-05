use crate::records::compiler::Compiler;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

const K_DEFAULT_ALLOC_PC: u32 = !0u32;

impl Compiler {
    pub fn ensure_export_table(&mut self, node: *mut AstNode) {
        self.exports.has_exports = true;

        let export_local = &mut self.exports.export_table_local as *mut _;
        if self.locals.contains(&export_local) {
            return;
        }

        LUAU_ASSERT!(self.at_top_level());

        let table_reg = self.alloc_reg(node, 1);
        unsafe {
            if luaur_common::FFlag::LuauOptimizeExportTable.get()
                && self.exports.exported_table_cid != -1
                && self.exports.exported_table_cid < 32768
            {
                (*self.bytecode).emit_ad(
                    LuauOpcode::LOP_DUPTABLE,
                    table_reg,
                    self.exports.exported_table_cid as i16,
                );
            } else {
                (*self.bytecode).emit_abc(
                    LuauOpcode::LOP_NEWTABLE,
                    table_reg,
                    Compiler::encode_hash_size(0),
                    0,
                );
                (*self.bytecode).emit_aux(0);
            }
        }

        self.push_local(export_local, table_reg, K_DEFAULT_ALLOC_PC);
    }
}
