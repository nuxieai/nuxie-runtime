use crate::records::ast_array::AstArray;
use crate::records::parser::Parser;

impl Parser {
    pub fn copy_t_usize<T: Clone>(&mut self, data: *const T, size: usize) -> AstArray<T> {
        let mut result = AstArray {
            data: core::ptr::null_mut(),
            size,
        };

        if size == 0 {
            return result;
        }

        unsafe {
            // allocator is a *mut Allocator in the Parser struct
            let storage = crate::records::allocator::Allocator::allocate(
                &mut *self.allocator,
                core::mem::size_of::<T>() * size,
            ) as *mut T;

            result.data = storage;

            // The C++ implementation uses placement new with the copy constructor.
            // In Rust, for types that are Clone, we can use ptr::write with clone().
            // Since Luau AST nodes/types in this context are POD-like or managed by the arena
            // and don't have destructors (as noted in the C++ comment), this is a safe mapping.
            for i in 0..size {
                let src = &*data.add(i);
                core::ptr::write(result.data.add(i), src.clone());
            }
        }

        result
    }
}
