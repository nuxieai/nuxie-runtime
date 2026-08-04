use crate::records::string_ref::StringRef;

impl StringRef {
    #[allow(non_snake_case)]
    pub(crate) fn operator_eq(&self, other: &StringRef) -> bool {
        if !self.data.is_null() && !other.data.is_null() {
            if self.length != other.length {
                return false;
            }
            if self.length == 0 {
                return true;
            }
            unsafe {
                core::ptr::eq(self.data, other.data)
                    || (core::slice::from_raw_parts(self.data as *const u8, self.length)
                        == core::slice::from_raw_parts(other.data as *const u8, other.length))
            }
        } else {
            self.data == other.data
        }
    }
}
