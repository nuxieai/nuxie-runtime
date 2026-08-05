use crate::records::compiler::Compiler;
use luaur_ast::records::ast_stat_block::AstStatBlock;
use luaur_ast::records::ast_stat_class::AstStatClass;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn preallocate_hoisted_classes(&mut self, body: *mut AstStatBlock) {
        unsafe {
            for i in 0..(*body).body.size {
                let stat = *(*body).body.data.add(i);
                let decl = luaur_ast::rtti::ast_node_as::<AstStatClass>(stat as *mut _);
                if !decl.is_null() {
                    let reg = self.alloc_reg(decl as *mut _, 1);
                    self.push_local((*decl).name, reg, !0u32);
                    (*self.bytecode).emit_abc(LuauOpcode::LOP_LOADNIL, reg, 0, 0);
                }
            }
        }
    }
}
