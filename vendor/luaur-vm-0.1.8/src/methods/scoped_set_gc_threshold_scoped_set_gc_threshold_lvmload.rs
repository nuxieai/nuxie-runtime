use crate::records::global_state::global_State;
use crate::records::scoped_set_gc_threshold::ScopedSetGcThreshold;

impl ScopedSetGcThreshold {
    pub fn scoped_set_gc_threshold_global_state_usize(
        &mut self,
        global: *mut global_State,
        new_threshold: usize,
    ) {
        self.global = global;
        unsafe {
            self.original_threshold = (*global).GCthreshold;
            (*global).GCthreshold = new_threshold;
        }
    }
}
