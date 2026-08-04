use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_successor(&mut self, from_op: BcOp, to_op: BcOp, kind: BcBlockEdgeKind) {
        let from: &mut BcBlock = self.func.block_op(from_op);
        from.successors.push_back(BcBlockEdge {
            kind,
            target: to_op,
        });

        let to: &mut BcBlock = self.func.block_op(to_op);
        to.predecessors.push_back(BcBlockEdge {
            kind,
            target: from_op,
        });
    }
}
