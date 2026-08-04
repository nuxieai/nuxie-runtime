use crate::records::bc_op::BcOp;
use crate::records::bc_op_hash::BcOpHash;
use crate::records::constness_lattice::ConstnessLattice;

pub type OpConstness =
    luaur_common::records::dense_hash_map2::DenseHashMap2<BcOp, ConstnessLattice, BcOpHash>;
