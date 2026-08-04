use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_load_nil::BcLoadNil;
use crate::records::bc_move::BcMove;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::records::bc_return::BcReturn;
use crate::records::call_inliner::CallInliner;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> CallInliner<'a> {
    pub fn replace_return(
        &mut self,
        next_block: &mut BcRef<'a, BcBlock>,
        caller_block_op: BcOp,
        target_return_op: BcOp,
    ) -> bool {
        let target = self.target as *mut crate::records::bc_function::BcFunction;
        let target_return_ref = unsafe { (&*target).inst(target_return_op) };
        let mut ret =
            BcReturn::<crate::records::bc_function::VmConst>::from(target, target_return_ref);
        let return_count = ret.return_count();
        if return_count < 0 {
            return false;
        }
        let values = ret.values();

        let mut i = 0;
        while i < values.len() as u32 {
            let src = self.map_to_caller_op(values[i as usize]);
            let mut move_op = BcMove::<crate::records::bc_function::VmConst>::create(self.caller);
            move_op.set_src(src);
            move_op.set_out_reg(self.target_reg + i as u8);
            move_op.append_to(caller_block_op);
            let op = move_op.op();
            drop(move_op);
            self.set_return_op(i, op);
            i += 1;
        }

        let call_res = self.call.return_count();
        LUAU_ASSERT!(call_res >= 0);
        let call_res = call_res as u32;

        while i < call_res {
            let mut load_nil =
                BcLoadNil::<crate::records::bc_function::VmConst>::create(self.caller);
            load_nil.set_out_reg(self.target_reg + i as u8);
            load_nil.append_to(caller_block_op);
            let op = load_nil.op();
            drop(load_nil);
            self.set_return_op(i, op);
            i += 1;
        }

        let mut caller_block = self.caller.block(caller_block_op);
        caller_block
            .operator_deref_mut()
            .successors
            .push_back(BcBlockEdge {
                kind: BcBlockEdgeKind::Fallthrough,
                target: next_block.op,
            });
        next_block
            .operator_deref_mut()
            .predecessors
            .push_back(BcBlockEdge {
                kind: BcBlockEdgeKind::Fallthrough,
                target: caller_block_op,
            });

        true
    }
}
