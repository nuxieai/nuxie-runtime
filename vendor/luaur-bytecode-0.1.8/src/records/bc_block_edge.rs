use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::records::bc_op::BcOp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BcBlockEdge {
    pub kind: BcBlockEdgeKind,
    pub target: BcOp,
}
