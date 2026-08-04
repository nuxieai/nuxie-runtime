use crate::records::bc_op::BcOp;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct BcRef<'a, T> {
    pub(crate) vec: &'a Vec<T>,
    pub(crate) op: BcOp,
}

impl<'a, T> Clone for BcRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for BcRef<'a, T> {}
