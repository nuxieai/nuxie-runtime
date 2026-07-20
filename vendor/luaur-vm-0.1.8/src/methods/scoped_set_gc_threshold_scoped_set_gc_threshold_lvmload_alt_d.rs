use crate::records::scoped_set_gc_threshold::ScopedSetGcThreshold;

impl ScopedSetGcThreshold {
    pub fn drop(&mut self) {
        unsafe {
            if !self.global.is_null() {
                (*self.global).GCthreshold = self.original_threshold;
            }
        }
    }
}
