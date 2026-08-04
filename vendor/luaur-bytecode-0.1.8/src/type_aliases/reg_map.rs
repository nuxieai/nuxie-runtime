use crate::records::bc_op::BcOp;
use crate::records::bc_op_hash::BcOpHash;
use crate::type_aliases::reg::Reg;
use std::collections::HashMap;

pub type RegMap = HashMap<BcOp, Reg, BcOpHash>;
