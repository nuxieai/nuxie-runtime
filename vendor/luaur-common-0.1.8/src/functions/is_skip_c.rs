use crate::enums::luau_opcode::LuauOpcode;

#[allow(non_snake_case)]
pub fn isSkipC(op: LuauOpcode) -> bool {
    match op {
        LuauOpcode::LOP_LOADB => true,

        _ => false,
    }
}
