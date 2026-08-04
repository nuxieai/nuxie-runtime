use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_ref::BcRef;
use crate::records::call_inliner::CallInliner;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> CallInliner<'a> {
    pub fn add_successor(
        &mut self,
        mut from: BcRef<'a, BcBlock>,
        mut to: BcRef<'a, BcBlock>,
        kind: BcBlockEdgeKind,
    ) {
        LUAU_ASSERT!(
            kind != BcBlockEdgeKind::Fallthrough
                || (!self.has_edge(
                    &from.operator_deref().successors,
                    BcBlockEdgeKind::Fallthrough
                ) && !self.has_edge(
                    &to.operator_deref().predecessors,
                    BcBlockEdgeKind::Fallthrough
                ))
        );

        let from_op = from.op;
        let to_op = to.op;

        from.operator_deref_mut().successors.push_back(BcBlockEdge {
            kind,
            target: to_op,
        });

        to.operator_deref_mut().predecessors.push_back(BcBlockEdge {
            kind,
            target: from_op,
        });
    }
}
