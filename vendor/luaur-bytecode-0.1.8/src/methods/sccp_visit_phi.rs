use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp::Sccp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn visit_phi(&mut self, phi_op: BcOp) {
        let (ops, uses) = {
            let phi = &self.func().phis[phi_op.index as usize];
            LUAU_ASSERT!(!phi.ops.is_empty());
            (phi.ops.clone(), phi.uses.clone())
        };
        let mut fold = ConstnessLattice::default();
        for op in ops.iter() {
            let lattice = self.state.operand_lattice(op);
            fold = lattice.merge(&fold);
        }
        let previous = *self.state.op_constness.get_or_insert(phi_op);
        if fold != previous {
            self.ssa_worklist.extend(uses);
        }
        *self.state.op_constness.get_or_insert(phi_op) = fold;
    }
}
