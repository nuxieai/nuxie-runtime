use crate::records::bc_op::BcOp;
use luaur_common::records::small_vector::SmallVector;

pub type BcOps = SmallVector<BcOp, 4>;
