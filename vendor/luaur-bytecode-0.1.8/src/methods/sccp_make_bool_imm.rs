use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::constness::Constness;
use crate::records::bc_imm::{BcImm, BcImmValue};
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp::Sccp;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn make_bool_imm(&self, value: bool) -> ConstnessLattice {
        ConstnessLattice::from_imm_const(
            Constness::ImmConstant,
            BcImm {
                kind: BcImmKind::Boolean,
                value: BcImmValue {
                    valueBoolean: value,
                },
            },
        )
    }
}
