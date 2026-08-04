use crate::records::call_inliner::CallInliner;

impl<'a> CallInliner<'a> {
    pub(crate) fn allocate_vm_consts(&mut self) {
        self.caller_vm_const_size_before_inline = self.caller.constants.len() as u32;
        let reserve_size =
            self.caller_vm_const_size_before_inline + self.target.constants.len() as u32;
        self.caller.constants.reserve(reserve_size as usize);
        for c in &self.target.constants {
            self.caller.constants.push(c.clone());
        }
    }
}
