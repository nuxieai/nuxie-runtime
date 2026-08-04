use crate::records::bc_block_edge::BcBlockEdge;
use luaur_common::records::small_vector::SmallVector;

pub type BcEdges = SmallVector<BcBlockEdge, 2>;
