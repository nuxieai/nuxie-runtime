use crate::enums::constness::Constness;
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp::Sccp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn make_constant_op(&mut self, lattice: ConstnessLattice) -> BcOp {
        if lattice.kind == Constness::VmConstant {
            let value =
                self.func().constants[lattice.vm_const.expect("VM constant").index as usize];
            self.func_mut().add_const(&value)
        } else if lattice.kind == Constness::ImmConstant {
            self.func_mut()
                .add_imm_value(&lattice.imm_const.expect("immediate constant"))
        } else {
            LUAU_ASSERT!(false, "makeConstantOp called on non-constant lattice value");
            BcOp::new()
        }
    }
}
