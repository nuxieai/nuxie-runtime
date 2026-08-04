use crate::records::call_inliner::CallInliner;

impl<'a> CallInliner<'a> {
    pub(crate) fn allocate_instructions(&mut self) {
        self.caller_inst_size_before_inline = self.caller.instructions.len() as u32;
        self.caller.instructions.resize(
            self.caller_inst_size_before_inline as usize + self.target.instructions.len(),
            Default::default(),
        );
    }
}
