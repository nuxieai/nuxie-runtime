use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;

impl BcFunction {
    pub fn add_use_inst(&mut self, inst_user: BcOp, used_op: BcOp) {
        self.inst_op(inst_user).ops.push_back(used_op);
        self.record_use(used_op, inst_user);
    }
}
