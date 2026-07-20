use crate::enums::luau_opcode::LuauOpcode;

#[allow(non_snake_case)]
pub fn isJumpD(op: LuauOpcode) -> bool {
    match op {
        LuauOpcode::LOP_JUMP
        | LuauOpcode::LOP_JUMPIF
        | LuauOpcode::LOP_JUMPIFNOT
        | LuauOpcode::LOP_JUMPIFEQ
        | LuauOpcode::LOP_JUMPIFLE
        | LuauOpcode::LOP_JUMPIFLT
        | LuauOpcode::LOP_JUMPIFNOTEQ
        | LuauOpcode::LOP_JUMPIFNOTLE
        | LuauOpcode::LOP_JUMPIFNOTLT
        | LuauOpcode::LOP_FORNPREP
        | LuauOpcode::LOP_FORNLOOP
        | LuauOpcode::LOP_FORGPREP
        | LuauOpcode::LOP_FORGLOOP
        | LuauOpcode::LOP_FORGPREP_INEXT
        | LuauOpcode::LOP_FORGPREP_NEXT
        | LuauOpcode::LOP_JUMPBACK
        | LuauOpcode::LOP_JUMPXEQKNIL
        | LuauOpcode::LOP_JUMPXEQKB
        | LuauOpcode::LOP_JUMPXEQKN
        | LuauOpcode::LOP_JUMPXEQKS
        | LuauOpcode::LOP_CMPPROTO => true,

        _ => false,
    }
}
