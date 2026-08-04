use crate::records::call_inliner::CallInliner;

impl<'a> CallInliner<'a> {
    pub(crate) fn allocate_up_values(&mut self) {
        self.caller_up_val_size_before_inline = self.caller.nups;
        self.caller.nups += self.target.nups;
    }
}
