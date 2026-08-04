use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::sccp_state::SccpState;

impl SccpState {
    pub fn unknown_condition_constness(
        &mut self,
        ops: impl IntoIterator<Item = BcOp>,
    ) -> Constness {
        for op in ops {
            if self.operand_lattice(&op).kind == Constness::NotAConstant {
                return Constness::NotAConstant;
            }
        }
        Constness::Undetermined
    }
}
