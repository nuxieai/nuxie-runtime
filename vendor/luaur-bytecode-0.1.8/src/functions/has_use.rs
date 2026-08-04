use crate::functions::count_uses::count_uses;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;

pub fn has_use(function: &BcFunction, def: BcOp, consumer: BcOp) -> bool {
    count_uses(function, def, consumer) > 0
}
