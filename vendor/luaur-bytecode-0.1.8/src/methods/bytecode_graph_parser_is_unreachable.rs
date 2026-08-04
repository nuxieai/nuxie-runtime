use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

pub fn bytecode_graph_parser_is_unreachable(
    self_: &mut BytecodeGraphParser<'_>,
    block_op: BcOp,
) -> bool {
    if block_op == self_.func.entry_block {
        return false;
    }

    let predecessors = self_.func.block_op(block_op).predecessors.clone();
    for pred in &predecessors {
        if pred.kind == BcBlockEdgeKind::Loop {
            continue;
        }
        if !bytecode_graph_parser_is_unreachable(self_, pred.target) {
            return false;
        }
    }

    true
}
