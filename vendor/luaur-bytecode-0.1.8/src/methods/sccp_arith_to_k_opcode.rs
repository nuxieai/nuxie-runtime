use crate::records::sccp::Sccp;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn arith_to_k_opcode(op: LuauOpcode) -> Option<LuauOpcode> {
        match op {
            LuauOpcode::LOP_ADD => Some(LuauOpcode::LOP_ADDK),
            LuauOpcode::LOP_SUB => Some(LuauOpcode::LOP_SUBK),
            LuauOpcode::LOP_MUL => Some(LuauOpcode::LOP_MULK),
            LuauOpcode::LOP_DIV => Some(LuauOpcode::LOP_DIVK),
            LuauOpcode::LOP_MOD => Some(LuauOpcode::LOP_MODK),
            LuauOpcode::LOP_POW => Some(LuauOpcode::LOP_POWK),
            _ => None,
        }
    }
}
