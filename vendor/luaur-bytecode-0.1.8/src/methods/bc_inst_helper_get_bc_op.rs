use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;

impl BcInstHelper<'_> {
    pub fn get_bc_op(&mut self, input_idx: u32) -> BcOp {
        let inst = self.operator_deref();
        if (input_idx as usize) >= inst.ops.len() {
            let inst_mut = unsafe {
                &mut *(self.inst.operator_arrow() as *mut crate::records::bc_inst::BcInst)
            };
            inst_mut.ops.resize((input_idx + 1) as u32);
        }
        self.operator_deref().ops[input_idx as usize]
    }
}
