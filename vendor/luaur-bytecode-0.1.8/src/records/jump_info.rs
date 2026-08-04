use crate::records::bc_op::BcOp;
use luaur_common::enums::luau_opcode::LuauOpcode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JumpInfo {
    pub(crate) op: LuauOpcode,
    pub(crate) instructionPC: u32,
    pub(crate) targetBlock: BcOp,
}
