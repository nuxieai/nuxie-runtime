use crate::records::call_inliner::CallInliner;

impl<'a> CallInliner<'a> {
    pub(crate) fn allocate_protos(&mut self) {
        self.caller_proto_size_before_inline = self.caller.protos.len() as u32;
        self.caller.protos.resize(
            self.caller_proto_size_before_inline as usize + self.target.protos.len(),
            0,
        );
    }
}
