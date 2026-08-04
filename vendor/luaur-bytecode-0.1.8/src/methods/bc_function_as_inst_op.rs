use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;

impl BcFunction {
    pub fn as_inst_op(&self, op: BcOp) -> *mut BcInst {
        if op.kind == BcOpKind::Inst {
            if (op.index as usize) < self.instructions.len() {
                &self.instructions[op.index as usize] as *const BcInst as *mut BcInst
            } else {
                core::ptr::null_mut()
            }
        } else {
            core::ptr::null_mut()
        }
    }
}
