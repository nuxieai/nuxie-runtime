use crate::enums::luau_opcode::LuauOpcode;

#[allow(non_snake_case)]
pub fn isFallthrough(op: LuauOpcode) -> bool {
    match op {
        LuauOpcode::LOP_RETURN
        | LuauOpcode::LOP_JUMP
        | LuauOpcode::LOP_JUMPBACK
        | LuauOpcode::LOP_JUMPX => false,
        _ => true,
    }
}

pub fn is_fallthrough(op: LuauOpcode) -> bool {
    isFallthrough(op)
}
