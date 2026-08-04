use crate::enums::constness::Constness;
use crate::records::constness_lattice::ConstnessLattice;

impl ConstnessLattice {
    pub fn merge(&self, other: &ConstnessLattice) -> ConstnessLattice {
        if self.kind == Constness::Undetermined {
            return *other;
        }
        if other.kind == Constness::Undetermined {
            return *self;
        }
        if self == other {
            return *self;
        }
        ConstnessLattice::from_kind(Constness::NotAConstant)
    }
}
