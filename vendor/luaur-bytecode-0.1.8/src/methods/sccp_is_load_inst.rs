use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn is_load_inst(&self, op: BcOp) -> bool {
        op.kind == BcOpKind::Inst
            && matches!(
                self.func().instructions[op.index as usize].op,
                LuauOpcode::LOP_LOADK
                    | LuauOpcode::LOP_LOADKX
                    | LuauOpcode::LOP_LOADN
                    | LuauOpcode::LOP_LOADB
                    | LuauOpcode::LOP_LOADNIL
            )
    }
}
