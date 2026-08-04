use crate::records::bc_op::BcOp;
use crate::type_aliases::bc_ops::BcOps;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BcPhi {
    pub ops: BcOps,
    pub uses: Vec<BcOp>,
}
