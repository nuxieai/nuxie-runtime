use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;
use crate::enums::bc_op_kind::BcOpKind;

impl BcInstHelper<'_> {
    pub(crate) fn set_bc_op(&mut self, input_idx: u32, op: BcOp) {
        if input_idx >= self.operator_deref().ops.len() as u32 {
            let inst_mut = unsafe {
                &mut *(self.inst.operator_arrow() as *mut crate::records::bc_inst::BcInst)
            };
            inst_mut.ops.resize(input_idx + 1);
        }
        self.operator_deref_mut().ops[input_idx as usize] = op;

        let user = self.inst.op;
        if op.kind == BcOpKind::Inst {
            let uses = &mut self.graph.instructions[op.index as usize].uses;
            if !uses.contains(&user) {
                uses.push(user);
            }
        } else if op.kind == BcOpKind::Phi {
            let uses = &mut self.graph.phis[op.index as usize].uses;
            if !uses.contains(&user) {
                uses.push(user);
            }
        }
    }

    pub(crate) fn operator_deref_mut(&mut self) -> &mut crate::records::bc_inst::BcInst {
        unsafe { &mut *self.inst.operator_arrow() }
    }
}
