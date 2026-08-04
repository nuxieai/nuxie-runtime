use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;

impl BcInstHelper<'_> {
    pub(crate) fn set_bc_op(&mut self, input_idx: u32, op: BcOp) {
        if input_idx >= self.operator_deref().ops.len() as u32 {
            let inst_mut = unsafe {
                &mut *(self.inst.operator_arrow() as *mut crate::records::bc_inst::BcInst)
            };
            inst_mut.ops.resize(input_idx + 1);
        }
        self.operator_deref_mut().ops[input_idx as usize] = op;
    }

    pub(crate) fn operator_deref_mut(&mut self) -> &mut crate::records::bc_inst::BcInst {
        unsafe { &mut *self.inst.operator_arrow() }
    }
}
