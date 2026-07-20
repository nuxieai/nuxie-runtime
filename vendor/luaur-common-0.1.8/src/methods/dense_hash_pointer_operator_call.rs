use crate::records::dense_hash_pointer::DenseHashPointer;

impl DenseHashPointer {
    #[allow(non_snake_case)]
    #[inline]
    pub fn operator_call(&self, key: *const core::ffi::c_void) -> usize {
        let addr = key as usize;
        (addr >> 4) ^ (addr >> 9)
    }
}
