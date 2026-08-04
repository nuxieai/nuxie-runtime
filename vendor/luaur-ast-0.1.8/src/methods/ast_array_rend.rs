use crate::records::ast_array::AstArray;

impl<T> AstArray<T> {
    pub fn rend(&self) -> core::iter::Rev<core::slice::Iter<'_, T>> {
        self.rbegin()
    }
}
