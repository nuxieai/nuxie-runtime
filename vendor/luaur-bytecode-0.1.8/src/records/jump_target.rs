use crate::enums::condition_state::ConditionState;
use crate::records::bc_op::BcOp;

#[derive(Debug, Clone, Copy)]
pub struct JumpTarget {
    pub dead: bool,
    pub block_op: BcOp,
    pub condition: ConditionState,
}

impl Default for JumpTarget {
    fn default() -> Self {
        Self {
            dead: false,
            block_op: BcOp::new(),
            condition: ConditionState::Unknown,
        }
    }
}
