use crate::records::ast_name::AstName;

impl AstName {
    #[allow(clippy::non_canonical_partial_ord_impl)]
    pub fn operator_lt(&self, rhs: &AstName) -> bool {
        if !self.value.is_null() && !rhs.value.is_null() {
            unsafe { core::ffi::CStr::from_ptr(self.value) < core::ffi::CStr::from_ptr(rhs.value) }
        } else {
            self.value < rhs.value
        }
    }
}
