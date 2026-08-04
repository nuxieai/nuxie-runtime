use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_op::BcOp;
use crate::records::call_inliner::CallInliner;
use crate::type_aliases::bc_edges::BcEdges;

impl<'a> CallInliner<'a> {
    pub fn set_fallthrough(&mut self, edges: &mut BcEdges, entry: BcOp) {
        for e in edges.iter_mut() {
            if e.kind == BcBlockEdgeKind::Fallthrough {
                e.target = entry;
                return;
            }
        }
        edges.push_back(BcBlockEdge {
            kind: BcBlockEdgeKind::Fallthrough,
            target: entry,
        });
    }
}
