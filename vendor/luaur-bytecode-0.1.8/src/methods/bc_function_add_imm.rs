use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;

impl BcFunction {
    pub fn add_imm(&mut self, kind: BcImmKind) -> BcOp {
        let imm = BcImm {
            kind,
            value: unsafe { core::mem::zeroed() },
        };
        self.immediates.push(imm);
        BcOp::bc_op_bc_op_kind_u32(BcOpKind::Imm, (self.immediates.len() - 1) as u32)
    }
}
