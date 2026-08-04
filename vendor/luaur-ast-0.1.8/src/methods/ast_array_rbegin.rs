use crate::records::ast_array::AstArray;

impl<T> AstArray<T> {
    pub fn rbegin(&self) -> core::iter::Rev<core::slice::Iter<'_, T>> {
        unsafe {
            core::slice::from_raw_parts(self.data, self.size)
                .iter()
                .rev()
        }
    }
}
