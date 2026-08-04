use crate::records::ast_array::AstArray;
use crate::records::parser::Parser;
use core::ffi::c_char;

impl Parser {
    pub fn copy_bytes(&mut self, data: &[u8]) -> AstArray<c_char> {
        let len = data.len();
        let mut result: AstArray<c_char> = AstArray {
            data: core::ptr::null_mut(),
            size: len,
        };

        unsafe {
            let storage =
                crate::records::allocator::Allocator::allocate(&mut *self.allocator, len + 1)
                    as *mut c_char;

            if len > 0 {
                core::ptr::copy_nonoverlapping(data.as_ptr() as *const c_char, storage, len);
            }
            *storage.add(len) = 0;

            result.data = storage;
        }

        result
    }
}
