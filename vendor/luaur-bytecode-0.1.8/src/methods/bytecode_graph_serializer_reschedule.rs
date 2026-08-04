use crate::enums::bc_block_flag::BcBlockFlag;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use luaur_common::LUAU_ASSERT;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn reschedule(&mut self) -> Vec<BcOp> {
        let mut sorted_blocks: Vec<BcOp> = Vec::new();
        sorted_blocks.reserve(self.func.blocks.len());
        for i in 0..self.func.blocks.len() {
            if (self.func.blocks[i as usize].flags & BcBlockFlag::Dead as u8) == 0 {
                sorted_blocks.push(BcOp::bc_op_bc_op_kind_u32(BcOpKind::Block, i as u32));
            }
        }

        sorted_blocks.sort_by(|op_a, op_b| {
            let a_block = self.func.block_op(*op_a);
            let a_sortkey = a_block.sortkey;
            let a_chainkey = a_block.chainkey;
            drop(a_block);

            let b_block = self.func.block_op(*op_b);
            let b_sortkey = b_block.sortkey;
            let b_chainkey = b_block.chainkey;
            drop(b_block);

            if a_sortkey == b_sortkey {
                a_chainkey.cmp(&b_chainkey)
            } else {
                a_sortkey.cmp(&b_sortkey)
            }
        });

        LUAU_ASSERT!(sorted_blocks.last() == Some(&self.func.exit_block));
        sorted_blocks.pop();

        sorted_blocks
    }
}
