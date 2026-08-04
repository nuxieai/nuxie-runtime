use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_imm::BcImm;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_imm_input_bc_inst_bool(&mut self, inst: *mut BcInst, value: bool) {
        let inst = unsafe { &mut *inst };
        let mut op = BcOp::bc_op_bc_op_kind_u32(BcOpKind::Imm, 0);
        let mut found = false;

        for idx in 0..self.func.immediates.len() {
            let imm = &self.func.immediates[idx];
            if imm.kind == BcImmKind::Boolean && unsafe { imm.value.valueBoolean } == value {
                op.index = idx as u32;
                found = true;
                break;
            }
        }

        if !found {
            let mut imm = BcImm {
                kind: BcImmKind::Boolean,
                value: unsafe { core::mem::zeroed() },
            };
            unsafe {
                imm.value.valueBoolean = value;
            }
            self.func.immediates.push(imm);
            op.index = (self.func.immediates.len() - 1) as u32;
        }

        inst.ops.push_back(op);
    }
}
