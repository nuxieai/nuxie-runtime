use crate::records::ast_name::AstName;

impl AstName {
    pub fn operator_ne_c_char(&self, rhs: *const core::ffi::c_char) -> bool {
        if self.value.is_null() {
            return true;
        }
        unsafe { core::ffi::CStr::from_ptr(self.value) != core::ffi::CStr::from_ptr(rhs) }
    }
}
