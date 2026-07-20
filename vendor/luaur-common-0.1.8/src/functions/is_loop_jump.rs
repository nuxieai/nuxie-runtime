use crate::enums::luau_opcode::LuauOpcode;

#[allow(non_snake_case)]
pub fn isLoopJump(op: LuauOpcode) -> bool {
    match op {
        LuauOpcode::LOP_JUMPBACK | LuauOpcode::LOP_FORGLOOP | LuauOpcode::LOP_FORNLOOP => true,
        _ => false,
    }
}

pub fn is_loop_jump(op: LuauOpcode) -> bool {
    isLoopJump(op)
}
