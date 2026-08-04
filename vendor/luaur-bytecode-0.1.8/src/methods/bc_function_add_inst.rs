use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl BcFunction {
    pub fn add_inst(&mut self) -> BcOp {
        self.instructions.push(BcInst {
            op: LuauOpcode::LOP_NOP,
            block: BcOp::new(),
            ops: Default::default(),
            lastUse: 0,
            useCount: 0,
            line: 0,
        });
        BcOp::bc_op_bc_op_kind_u32(BcOpKind::Inst, (self.instructions.len() - 1) as u32)
    }
}
