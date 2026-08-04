use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn get_fallthrough(&self, block: &BcBlock) -> Option<BcOp> {
        for edge in &block.successors {
            if edge.kind == BcBlockEdgeKind::Fallthrough {
                return Some(edge.target);
            }
        }
        None
    }
}
