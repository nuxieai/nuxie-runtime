use crate::enums::luau_opcode::LuauOpcode;

#[allow(non_snake_case)]
pub fn getOpLength(op: LuauOpcode) -> i32 {
    match op {
        LuauOpcode::LOP_GETGLOBAL
        | LuauOpcode::LOP_SETGLOBAL
        | LuauOpcode::LOP_GETIMPORT
        | LuauOpcode::LOP_GETTABLEKS
        | LuauOpcode::LOP_SETTABLEKS
        | LuauOpcode::LOP_NAMECALL
        | LuauOpcode::LOP_JUMPIFEQ
        | LuauOpcode::LOP_JUMPIFLE
        | LuauOpcode::LOP_JUMPIFLT
        | LuauOpcode::LOP_JUMPIFNOTEQ
        | LuauOpcode::LOP_JUMPIFNOTLE
        | LuauOpcode::LOP_JUMPIFNOTLT
        | LuauOpcode::LOP_NEWTABLE
        | LuauOpcode::LOP_SETLIST
        | LuauOpcode::LOP_FORGLOOP
        | LuauOpcode::LOP_LOADKX
        | LuauOpcode::LOP_FASTCALL2
        | LuauOpcode::LOP_FASTCALL2K
        | LuauOpcode::LOP_FASTCALL3
        | LuauOpcode::LOP_JUMPXEQKNIL
        | LuauOpcode::LOP_JUMPXEQKB
        | LuauOpcode::LOP_JUMPXEQKN
        | LuauOpcode::LOP_JUMPXEQKS
        | LuauOpcode::LOP_GETUDATAKS
        | LuauOpcode::LOP_SETUDATAKS
        | LuauOpcode::LOP_NAMECALLUDATA
        | LuauOpcode::LOP_NEWCLASSMEMBER
        | LuauOpcode::LOP_CALLFB
        | LuauOpcode::LOP_CMPPROTO => 2,

        _ => 1,
    }
}

pub fn get_op_length(op: LuauOpcode) -> i32 {
    getOpLength(op)
}
