use crate::records::ast_array::AstArray;
use crate::records::parser::Parser;
use alloc::string::String;

impl Parser {
    pub fn copy_string(&mut self, data: &String) -> AstArray<core::ffi::c_char> {
        // C++ `copy(data.c_str(), data.size() + 1)` reads size()+1 bytes because
        // std::string::c_str() is NUL-terminated with a readable size()+1 buffer.
        // Rust's `String` is NOT NUL-terminated and `String::as_ptr()` is a dangling
        // pointer when the string is empty, so copying `len()+1` bytes from the source
        // over-reads it (a guaranteed SIGSEGV on empty content, UB otherwise). Allocate
        // len+1, copy the `len` content bytes, and write the trailing NUL ourselves so
        // the result keeps c_str() semantics (NUL-terminated, logical size = len).
        let len = data.len();
        let mut result: AstArray<core::ffi::c_char> = AstArray {
            data: core::ptr::null_mut(),
            size: len,
        };

        unsafe {
            let storage =
                crate::records::allocator::Allocator::allocate(&mut *self.allocator, len + 1)
                    as *mut core::ffi::c_char;

            if len > 0 {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr() as *const core::ffi::c_char,
                    storage,
                    len,
                );
            }
            *storage.add(len) = 0;

            result.data = storage;
        }

        result
    }
}
