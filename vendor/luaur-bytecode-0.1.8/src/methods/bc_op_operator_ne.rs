use crate::records::bc_op::BcOp;

impl BcOp {
    #[inline]
    pub fn operator_ne(&self, rhs: &BcOp) -> bool {
        !self.eq(rhs)
    }
}
