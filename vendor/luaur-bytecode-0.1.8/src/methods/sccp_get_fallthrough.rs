use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn get_fallthrough(&self, block_op: BcOp) -> Option<BcOp> {
        let block = &self.func().blocks[block_op.index as usize];
        let mut fallthrough = None;
        for successor in block.successors.iter() {
            if successor.kind == BcBlockEdgeKind::Fallthrough {
                if fallthrough.is_some() {
                    LUAU_ASSERT!(false, "Multiple fallthroughs");
                    return None;
                }
                fallthrough = Some(successor.target);
            }
        }
        fallthrough
    }
}
