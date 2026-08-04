use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp::Sccp;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn rewrite_to_load(&mut self, op: BcOp, lattice: ConstnessLattice) {
        let used_ops = self.func().instructions[op.index as usize].ops.clone();
        for used_op in used_ops.iter().copied() {
            self.erase_use(op, used_op);
        }
        let new_operand = if lattice.kind == Constness::VmConstant {
            lattice.vm_const.expect("VM constant lattice")
        } else {
            self.func_mut()
                .add_imm_value(&lattice.imm_const.expect("immediate lattice"))
        };
        let inst = &mut self.func_mut().instructions[op.index as usize];
        inst.ops.clear();
        if lattice.kind == Constness::VmConstant {
            inst.op = LuauOpcode::LOP_LOADK;
        } else {
            let imm = lattice.imm_const.expect("immediate lattice");
            inst.op = if imm.kind == BcImmKind::Boolean {
                LuauOpcode::LOP_LOADB
            } else {
                LuauOpcode::LOP_LOADN
            };
        }
        inst.ops.push(new_operand);
    }
}
