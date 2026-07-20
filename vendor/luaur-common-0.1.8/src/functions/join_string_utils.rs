use crate::functions::join_impl::joinImpl;
use alloc::string::String;
use alloc::vec::Vec;

#[allow(non_snake_case)]
pub fn join(segments: &Vec<&str>, delimiter: &str) -> String {
    joinImpl(segments, delimiter)
}

// Pinned overload name advertised by the dependency cards.
#[allow(unused_imports, non_snake_case)]
pub use join as join_vector_string_view_string_view;
