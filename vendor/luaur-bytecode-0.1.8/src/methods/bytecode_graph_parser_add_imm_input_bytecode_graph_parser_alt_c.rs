use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_imm_input_bc_inst_u32(&mut self, inst: BcOp, value: u32) {
        let mut op = BcOp::bc_op_bc_op_kind_u32(BcOpKind::Imm, 0);

        self.func.immediates.push(BcImm {
            kind: BcImmKind::Import,
            value: unsafe { core::mem::zeroed() },
        });

        if let Some(last) = self.func.immediates.last_mut() {
            unsafe {
                last.value.valueImport = value;
            }
        }

        op.index = (self.func.immediates.len() - 1) as u32;
        self.func.add_use_inst(inst, op);
    }
}
