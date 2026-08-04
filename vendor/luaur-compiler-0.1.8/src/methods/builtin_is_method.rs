use crate::records::builtin::Builtin;

impl Builtin {
    pub fn is_method(&self, table: &str, name: &str) -> bool {
        unsafe {
            if self.object.value.is_null() || self.method.value.is_null() {
                return false;
            }
            let obj_bytes = core::ffi::CStr::from_ptr(self.object.value).to_bytes();
            let method_bytes = core::ffi::CStr::from_ptr(self.method.value).to_bytes();
            obj_bytes == table.as_bytes() && method_bytes == name.as_bytes()
        }
    }
}
