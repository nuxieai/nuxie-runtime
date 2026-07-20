use crate::records::scoped_set_gc_threshold::ScopedSetGcThreshold;

impl ScopedSetGcThreshold {
    pub fn scoped_set_gc_threshold_scoped_set_gc_threshold(
        &mut self,
        _other: &ScopedSetGcThreshold,
    ) {
        let global = self.global;
        self.scoped_set_gc_threshold_global_state_usize(global, self.original_threshold);
        self.scoped_set_gc_threshold_scoped_set_gc_threshold_mut(ScopedSetGcThreshold {
            global: self.global,
            original_threshold: self.original_threshold,
        });
    }
}
