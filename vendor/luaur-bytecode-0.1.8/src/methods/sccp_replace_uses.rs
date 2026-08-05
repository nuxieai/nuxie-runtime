use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp::Sccp;
use alloc::vec::Vec;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::is_jump_d::isJumpD;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn replace_uses(&mut self) {
        let constants: Vec<(BcOp, ConstnessLattice)> = self
            .state
            .op_constness
            .iter()
            .map(|(op, lattice)| (*op, *lattice))
            .collect();
        for (op, lattice) in constants {
            if !matches!(lattice.kind, Constness::ImmConstant | Constness::VmConstant)
                || op.kind != BcOpKind::Inst
                || self.is_load_inst(op)
            {
                continue;
            }
            let opcode = self.func().instructions[op.index as usize].op;
            LUAU_ASSERT!(opcode != LuauOpcode::LOP_JUMPX);
            if isJumpD(opcode) {
                self.remove_dead_edges(op);
                self.func_mut().erase_op(op);
            } else {
                self.rewrite_to_load(op, lattice);
            }
        }
    }
}
