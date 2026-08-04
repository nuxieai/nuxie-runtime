use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;
use crate::type_aliases::reg::Reg;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn register_of(&self, op: BcOp) -> Option<Reg> {
        if op.kind == BcOpKind::VmReg {
            Some(op.index as Reg)
        } else {
            self.func().regs.get(&op).copied()
        }
    }
}
