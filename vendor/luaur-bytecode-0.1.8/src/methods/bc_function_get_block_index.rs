use crate::records::bc_block::BcBlock;
use crate::records::bc_function::BcFunction;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn get_block_index(&self, block: &BcBlock) -> u32 {
        // Can only be called with blocks from our vector
        let data_ptr = self.blocks.as_ptr();
        let block_ptr = block as *const BcBlock;

        unsafe {
            let size = self.blocks.len();
            let end_ptr = data_ptr.add(size);

            LUAU_ASSERT!(block_ptr >= data_ptr && block_ptr <= end_ptr);

            block_ptr.offset_from(data_ptr) as u32
        }
    }
}
