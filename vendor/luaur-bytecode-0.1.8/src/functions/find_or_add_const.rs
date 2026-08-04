use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const::BcVmConst;

pub fn find_or_add_const(func: &mut BcFunction, value: &BcVmConst) -> BcOp {
    for (i, constant) in func.constants.iter().enumerate() {
        if constant == value {
            return BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmConst, i as u32);
        }
    }

    func.add_const(value)
}
