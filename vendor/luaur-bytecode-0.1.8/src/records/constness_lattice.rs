use crate::enums::constness::Constness;
use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstnessLattice {
    pub kind: Constness,
    pub vm_const: Option<BcOp>,
    pub imm_const: Option<BcImm>,
}

impl Default for ConstnessLattice {
    fn default() -> Self {
        Self {
            kind: Constness::Undetermined,
            vm_const: None,
            imm_const: None,
        }
    }
}

impl luaur_common::records::dense_hash_table::DenseDefault for ConstnessLattice {
    fn dense_default() -> Self {
        Self::default()
    }
}
