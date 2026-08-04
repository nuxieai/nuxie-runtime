use crate::records::bc_op::BcOp;

impl BcOp {
    #[inline]
    pub fn operator_eq(&self, rhs: &BcOp) -> bool {
        self.kind == rhs.kind && self.index == rhs.index
    }
}
