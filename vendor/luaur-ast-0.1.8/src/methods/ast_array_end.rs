use crate::records::ast_array::AstArray;

impl<T> AstArray<T> {
    pub fn end(&self) -> *const T {
        if self.data.is_null() {
            core::ptr::null()
        } else {
            unsafe { self.data.add(self.size) }
        }
    }
}
