use crate::records::bc_op::BcOp;
use crate::type_aliases::bc_ops::BcOps;
use luaur_common::enums::luau_opcode::LuauOpcode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BcInst {
    pub op: LuauOpcode,
    pub block: BcOp,
    pub ops: BcOps,
    pub lastUse: u32,
    pub useCount: u32,
    pub line: u32,
}

impl Default for BcInst {
    fn default() -> Self {
        Self {
            op: LuauOpcode::LOP_NOP,
            block: BcOp::new(),
            ops: Default::default(),
            lastUse: 0,
            useCount: 0,
            line: 0,
        }
    }
}
