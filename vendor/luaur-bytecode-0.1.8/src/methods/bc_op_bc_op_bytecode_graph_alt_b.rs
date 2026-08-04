use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;

impl BcOp {
    pub fn bc_op_bc_op_kind_u32(kind: BcOpKind, index: u32) -> Self {
        Self { kind, index }
    }
}
