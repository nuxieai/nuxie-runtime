use crate::records::call_inliner::CallInliner;

impl<'a> CallInliner<'a> {
    pub(crate) fn allocate_graph_entities_for_target(&mut self) {
        self.allocate_blocks();
        self.allocate_instructions();
        self.allocate_vm_consts();
        self.allocate_protos();
        self.allocate_up_values();
    }
}
