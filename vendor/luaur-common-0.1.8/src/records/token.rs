#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    pub(crate) name: *const core::ffi::c_char,
    pub(crate) category: *const core::ffi::c_char,
}

unsafe impl Send for Token {}
unsafe impl Sync for Token {}

impl Default for Token {
    fn default() -> Self {
        Self {
            name: core::ptr::null(),
            category: core::ptr::null(),
        }
    }
}
