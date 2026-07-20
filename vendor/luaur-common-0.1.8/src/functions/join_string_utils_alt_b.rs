use crate::functions::join_impl::joinImpl;
use alloc::string::String;
use alloc::vec::Vec;

pub fn join(segments: &Vec<String>, delimiter: &str) -> String {
    joinImpl(segments, delimiter)
}

// Pinned overload name advertised by the dependency cards.
#[allow(unused_imports, non_snake_case)]
pub use join as join_vector_string_string_view;
