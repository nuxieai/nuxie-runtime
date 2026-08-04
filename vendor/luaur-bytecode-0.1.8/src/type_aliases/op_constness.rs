use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;

pub type OpConstness = std::collections::HashMap<BcOp, ConstnessLattice>;
