use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_imm::BcImm;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_imm_input_bc_inst_i32(&mut self, inst: *mut BcInst, value: i32) {
        let inst = unsafe { &mut *inst };
        let mut op = BcOp::bc_op_bc_op_kind_u32(BcOpKind::Imm, 0);
        let mut i = 0usize;

        for idx in 0..self.func.immediates.len() {
            let imm = &self.func.immediates[idx];
            if imm.kind == BcImmKind::Int && unsafe { imm.value.valueInt } == value {
                op.index = idx as u32;
                i = idx;
                break;
            }
            i = idx + 1;
        }

        if i == self.func.immediates.len() {
            self.func.immediates.push(BcImm {
                kind: BcImmKind::Int,
                value: unsafe { core::mem::zeroed() },
            });
            if let Some(last) = self.func.immediates.last_mut() {
                unsafe {
                    last.value.valueInt = value;
                }
            }
            op.index = (self.func.immediates.len() - 1) as u32;
        }

        inst.ops.push_back(op);
    }
}
