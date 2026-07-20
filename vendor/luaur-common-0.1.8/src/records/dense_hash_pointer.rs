#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DenseHashPointer;

impl DenseHashPointer {
    #[inline]
    pub fn hash(&self, key: *const core::ffi::c_void) -> usize {
        let addr = key as usize;
        (addr >> 4) ^ (addr >> 9)
    }
}

impl DenseHashPointer {
    #[allow(non_snake_case)]
    #[inline]
    pub fn call(&self, key: *const core::ffi::c_void) -> usize {
        self.hash(key)
    }
}
