use crate::records::bc_block::BcBlock;
use crate::records::bc_inst::BcInst;
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_ref::BcRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcInstHelper<'_> {
    pub fn insert_before(&mut self, op: BcRef<'_, BcInst>) {
        let op_block = unsafe { (*op.operator_arrow()).block };

        // inst->block = op->block;
        unsafe {
            (*self.inst.operator_arrow()).block = op_block;
        }

        // BcRef<BcBlock> block = graph.block(op->block);
        let block_ref = self.graph.block(op_block);
        let block: &mut BcBlock = unsafe { &mut *block_ref.operator_arrow() };

        // auto it = std::find(block->ops.begin(), block->ops.end(), op.op);
        let it = block.ops.iter().position(|&x| x == op.op);

        // LUAU_ASSERT(it != block->ops.end());
        LUAU_ASSERT!(it.is_some());

        // block->ops.insert(it, inst.op);
        if let Some(index) = it {
            block.ops.insert(index, self.inst.op);
        }
    }
}
