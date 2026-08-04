use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_cmp_proto::BcCmpProto;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::records::call_inliner::CallInliner;

impl<'a> CallInliner<'a> {
    pub fn append_cmp_proto(
        &mut self,
        prev_block: &mut BcRef<'a, BcBlock>,
        target_op: BcOp,
        target_proto_id: u32,
    ) {
        let call_block = self.call.base.operator_deref().block;
        {
            let mut cmp_proto =
                BcCmpProto::<crate::records::bc_function::VmConst>::create(self.caller);
            cmp_proto.set_closure(target_op);
            cmp_proto.set_proto_id(target_proto_id);
            cmp_proto.set_fallback(call_block);
            cmp_proto.append_to(prev_block.op);
        }
        prev_block
            .operator_deref_mut()
            .successors
            .push_back(BcBlockEdge {
                kind: BcBlockEdgeKind::Branch,
                target: call_block,
            });
        self.caller.blocks[call_block.index as usize]
            .predecessors
            .push_back(BcBlockEdge {
                kind: BcBlockEdgeKind::Branch,
                target: prev_block.op,
            });
    }
}
