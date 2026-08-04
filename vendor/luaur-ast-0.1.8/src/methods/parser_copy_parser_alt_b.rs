use crate::records::ast_array::AstArray;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    #[allow(non_snake_case)]
    pub fn copy_temp_vector_t<'a, T: Clone>(&mut self, data: &TempVector<'a, T>) -> AstArray<T> {
        if data.size_ == 0 {
            self.copy_t_usize(core::ptr::null_mut(), 0)
        } else {
            unsafe {
                let ptr = (*data.storage).as_ptr().add(data.offset);
                self.copy_t_usize(ptr as *mut T, data.size_)
            }
        }
    }
}
