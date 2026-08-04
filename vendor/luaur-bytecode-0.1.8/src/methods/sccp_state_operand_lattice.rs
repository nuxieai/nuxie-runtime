use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp_state::SccpState;

impl SccpState {
    pub fn operand_lattice(&mut self, op: &BcOp) -> ConstnessLattice {
        if matches!(
            op.kind,
            BcOpKind::Proj | BcOpKind::VmReg | BcOpKind::VmUpvalue
        ) {
            return ConstnessLattice::from_kind(Constness::NotAConstant);
        }

        *self.op_constness.entry(*op).or_default()
    }
}
