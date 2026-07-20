use luaur_common::enums::luau_opcode::LuauOpcode;

pub fn luau_skipstep(op: u8) -> bool {
    op == LuauOpcode::LOP_PREPVARARGS as u8 || op == LuauOpcode::LOP_BREAK as u8
}
