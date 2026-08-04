#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DenseHashPointer;

impl DenseHashPointer {
    #[inline]
    pub fn hash(&self, key: *const core::ffi::c_void) -> usize {
        let mut u = key as usize as u64;
        u = u.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        u ^= u >> 31;
        u as usize
    }
}

impl DenseHashPointer {
    #[allow(non_snake_case)]
    #[inline]
    pub fn call(&self, key: *const core::ffi::c_void) -> usize {
        self.hash(key)
    }
}
