use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;

impl BcFunction {
    pub fn set_ops(&mut self, op: BcOp, new_ops: &[BcOp]) {
        let old_ops = self.instructions[op.index as usize].ops.clone();
        for old_op in old_ops.iter().copied() {
            self.erase_use(op, old_op);
        }

        self.instructions[op.index as usize].ops.clear();
        for &new_op in new_ops {
            self.instructions[op.index as usize].ops.push(new_op);
            self.record_use(new_op, op);
        }
    }
}
