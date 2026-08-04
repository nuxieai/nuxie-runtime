use crate::records::ast_array::AstArray;
use crate::records::parser::Parser;

#[allow(non_snake_case)]
impl Parser {
    pub fn copy_initializer_list_t<T: Clone>(&mut self, data: &[T]) -> AstArray<T> {
        self.copy_t_usize(
            if data.is_empty() {
                core::ptr::null()
            } else {
                data.as_ptr()
            },
            data.len(),
        )
    }
}
