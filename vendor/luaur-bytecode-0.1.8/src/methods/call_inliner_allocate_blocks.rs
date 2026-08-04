use crate::records::call_inliner::CallInliner;

impl<'a> CallInliner<'a> {
    pub(crate) fn allocate_blocks(&mut self) {
        self.caller_blocks_size_before_inline = self.caller.blocks.len() as u32;
        self.caller.blocks.resize(
            self.caller_blocks_size_before_inline as usize + self.target.blocks.len(),
            Default::default(),
        );
    }
}
