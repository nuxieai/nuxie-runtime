use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp::Sccp;
use luaur_common::enums::luau_capture_type::LuauCaptureType;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn visit_inst(&mut self, inst_op: BcOp) {
        let (opcode, ops, uses, block) = {
            let inst = &self.func().instructions[inst_op.index as usize];
            (inst.op, inst.ops.clone(), inst.uses.clone(), inst.block)
        };

        if opcode == LuauOpcode::LOP_CAPTURE && ops.len() >= 2 {
            LUAU_ASSERT!(ops[0].kind == BcOpKind::Imm);
            let capture = self.func().immediates[ops[0].index as usize];
            if capture.kind == BcImmKind::Int
                && unsafe { capture.value.valueInt } == LuauCaptureType::LCT_REF as i32
            {
                let source = ops[1];
                let previous = *self.state.op_constness.get_or_insert(source);
                if previous.kind != Constness::NotAConstant
                    && matches!(source.kind, BcOpKind::Inst | BcOpKind::Phi)
                {
                    *self.state.op_constness.get_or_insert(source) =
                        ConstnessLattice::from_kind(Constness::NotAConstant);
                    let source_uses = if source.kind == BcOpKind::Inst {
                        self.func().instructions[source.index as usize].uses.clone()
                    } else {
                        self.func().phis[source.index as usize].uses.clone()
                    };
                    self.ssa_worklist.extend(source_uses);
                }
            }
        }

        let lattice = self
            .interpreter
            .evaluate(opcode, unsafe { &*self.func }, inst_op);
        let previous = *self.state.op_constness.get_or_insert(inst_op);
        let new_value = lattice.merge(&previous);
        if new_value != previous {
            self.ssa_worklist.extend(uses);
        }

        for target in self.jump_targets(inst_op) {
            let block_index = target.block_op.index;
            if !target.dead {
                self.block_uses.get_or_insert(block_index).insert(block);
                if !self.flow_worklist_set.contains(&target.block_op) {
                    self.flow_worklist.push_back(target.block_op);
                }
            }
        }
        *self.state.op_constness.get_or_insert(inst_op) = new_value;
    }
}
