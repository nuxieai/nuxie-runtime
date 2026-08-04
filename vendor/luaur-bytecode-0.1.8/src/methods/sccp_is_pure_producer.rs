use crate::records::sccp::Sccp;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn is_pure_producer(&self, op: LuauOpcode) -> bool {
        matches!(
            op,
            LuauOpcode::LOP_LOADK
                | LuauOpcode::LOP_LOADKX
                | LuauOpcode::LOP_LOADN
                | LuauOpcode::LOP_LOADB
                | LuauOpcode::LOP_LOADNIL
                | LuauOpcode::LOP_GETUPVAL
        )
    }
}
