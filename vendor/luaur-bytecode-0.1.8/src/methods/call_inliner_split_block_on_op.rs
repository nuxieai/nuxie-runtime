use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::records::call_inliner::CallInliner;
use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> CallInliner<'a> {
    pub fn split_block_on_op(
        &mut self,
        split_op: BcOp,
    ) -> (BcRef<'a, BcBlock>, BcRef<'a, BcBlock>) {
        let prev_block_op = self.caller.instructions[split_op.index as usize].block;
        LUAU_ASSERT!(prev_block_op.kind == BcOpKind::Block);
        LUAU_ASSERT!(self.caller.blocks[prev_block_op.index as usize]
            .ops
            .iter()
            .any(|op| *op == split_op));

        let insn_block_op = self.caller.add_block();
        let prev_block_sortkey = self.caller.blocks[prev_block_op.index as usize].sortkey;
        let prev_block_chainkey = self.caller.blocks[prev_block_op.index as usize].chainkey;
        self.caller.blocks[insn_block_op.index as usize].sortkey = prev_block_sortkey;
        self.caller.blocks[insn_block_op.index as usize].chainkey = prev_block_chainkey + 1;

        let next_block_op = self.caller.add_block();
        self.caller.blocks[next_block_op.index as usize].sortkey =
            self.caller.blocks[insn_block_op.index as usize].sortkey;
        self.caller.blocks[next_block_op.index as usize].chainkey =
            self.caller.blocks[insn_block_op.index as usize].chainkey + 1;

        while *self.caller.blocks[prev_block_op.index as usize]
            .ops
            .back()
            .unwrap()
            != split_op
        {
            let back_op = *self.caller.blocks[prev_block_op.index as usize]
                .ops
                .back()
                .unwrap();
            self.caller.blocks[prev_block_op.index as usize]
                .ops
                .pop_back();
            self.caller.blocks[next_block_op.index as usize]
                .ops
                .push_front(back_op);
            self.caller.instructions[back_op.index as usize].block = next_block_op;
        }

        let prev_successors = self.caller.blocks[prev_block_op.index as usize]
            .successors
            .clone();
        for e in &prev_successors {
            let succ = e.target;
            for pred in self.caller.blocks[succ.index as usize]
                .predecessors
                .iter_mut()
            {
                if pred.target == prev_block_op {
                    pred.target = next_block_op;
                }
            }
        }
        self.caller.blocks[next_block_op.index as usize].successors = prev_successors;
        self.caller.blocks[prev_block_op.index as usize]
            .successors
            .clear();

        self.caller.blocks[prev_block_op.index as usize]
            .successors
            .push_back(BcBlockEdge {
                kind: BcBlockEdgeKind::Fallthrough,
                target: insn_block_op,
            });
        self.caller.blocks[insn_block_op.index as usize]
            .predecessors
            .push_back(BcBlockEdge {
                kind: BcBlockEdgeKind::Fallthrough,
                target: prev_block_op,
            });
        self.caller.blocks[insn_block_op.index as usize]
            .successors
            .push_back(BcBlockEdge {
                kind: BcBlockEdgeKind::Fallthrough,
                target: next_block_op,
            });
        self.caller.blocks[next_block_op.index as usize]
            .predecessors
            .push_back(BcBlockEdge {
                kind: BcBlockEdgeKind::Fallthrough,
                target: insn_block_op,
            });

        LUAU_ASSERT!(
            *self.caller.blocks[prev_block_op.index as usize]
                .ops
                .back()
                .unwrap()
                == split_op
        );
        self.caller.blocks[prev_block_op.index as usize]
            .ops
            .pop_back();
        self.caller.blocks[insn_block_op.index as usize]
            .ops
            .push_back(split_op);
        self.caller.instructions[split_op.index as usize].block = insn_block_op;

        let vec_ptr: *const Vec<BcBlock> = &self.caller.blocks;
        let vec_ref: &'a Vec<BcBlock> = unsafe { &*vec_ptr };
        (
            BcRef {
                vec: vec_ref,
                op: prev_block_op,
            },
            BcRef {
                vec: vec_ref,
                op: next_block_op,
            },
        )
    }
}
