use crate::records::scoped_set_gc_threshold::ScopedSetGcThreshold;

impl ScopedSetGcThreshold {
    pub fn operator_assign_mut(&mut self, _other: ScopedSetGcThreshold) {
        panic!("ScopedSetGcThreshold::operator= is deleted");
    }
}
