use crate::records::bc_op::BcOp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BcProj {
    pub op: BcOp,
    pub index: u32,
}
