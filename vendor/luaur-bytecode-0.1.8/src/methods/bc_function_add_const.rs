use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::{BcFunction, VmConst};
use crate::records::bc_op::BcOp;

impl BcFunction {
    pub fn add_const(&mut self, value: &VmConst) -> BcOp {
        self.constants.push(*value);
        BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmConst, (self.constants.len() - 1) as u32)
    }
}
