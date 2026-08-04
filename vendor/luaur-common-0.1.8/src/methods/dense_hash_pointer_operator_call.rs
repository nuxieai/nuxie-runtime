use crate::records::dense_hash_pointer::DenseHashPointer;

impl DenseHashPointer {
    #[allow(non_snake_case)]
    #[inline]
    pub fn operator_call(&self, key: *const core::ffi::c_void) -> usize {
        let mut u = key as usize as u64;
        u = u.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        u ^= u >> 31;
        u as usize
    }
}
