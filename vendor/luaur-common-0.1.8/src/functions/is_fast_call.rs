use crate::enums::luau_opcode::LuauOpcode;

#[allow(non_snake_case)]
pub fn isFastCall(op: LuauOpcode) -> bool {
    match op {
        LuauOpcode::LOP_FASTCALL
        | LuauOpcode::LOP_FASTCALL1
        | LuauOpcode::LOP_FASTCALL2
        | LuauOpcode::LOP_FASTCALL2K
        | LuauOpcode::LOP_FASTCALL3 => true,

        _ => false,
    }
}

pub fn is_fast_call(op: LuauOpcode) -> bool {
    isFastCall(op)
}
