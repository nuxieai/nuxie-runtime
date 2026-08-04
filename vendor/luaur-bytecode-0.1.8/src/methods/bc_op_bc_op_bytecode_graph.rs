use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;

impl BcOp {
    pub fn new() -> Self {
        Self {
            kind: BcOpKind::None,
            index: 0,
        }
    }
}

impl Default for BcOp {
    fn default() -> Self {
        Self::new()
    }
}
