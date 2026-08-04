use crate::records::ast_array::AstArray;

impl<T> AstArray<T> {
    pub fn begin(&self) -> *const T {
        self.data
    }
}
