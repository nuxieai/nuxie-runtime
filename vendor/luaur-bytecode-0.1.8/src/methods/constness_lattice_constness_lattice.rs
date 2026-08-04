use crate::enums::constness::Constness;
use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl ConstnessLattice {
    pub fn from_vm_const(kind: Constness, bc_op: BcOp) -> Self {
        LUAU_ASSERT!(kind == Constness::VmConstant);
        Self {
            kind,
            vm_const: Some(bc_op),
            imm_const: None,
        }
    }

    pub fn from_imm_const(kind: Constness, imm: BcImm) -> Self {
        LUAU_ASSERT!(kind == Constness::ImmConstant);
        Self {
            kind,
            vm_const: None,
            imm_const: Some(imm),
        }
    }

    pub fn from_kind(kind: Constness) -> Self {
        Self {
            kind,
            vm_const: None,
            imm_const: None,
        }
    }
}
