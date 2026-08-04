use crate::functions::uses_of::uses_of;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;

pub fn count_uses(function: &BcFunction, def: BcOp, consumer: BcOp) -> i32 {
    uses_of(function, def)
        .iter()
        .filter(|use_op| **use_op == consumer)
        .count() as i32
}
