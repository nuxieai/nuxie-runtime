use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;

use crate::records::bc_op::BcOp;
use crate::records::bc_phi::BcPhi;

impl BcFunction {
    pub fn add_phi(&mut self) -> BcOp {
        self.phis.push(BcPhi {
            ops: Default::default(),
        });
        BcOp::bc_op_bc_op_kind_u32(BcOpKind::Phi, (self.phis.len() - 1) as u32)
    }
}
