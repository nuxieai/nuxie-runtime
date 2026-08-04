use crate::records::allocator::Allocator;

#[allow(non_snake_case)]
impl Allocator {
    // The C++ source defines `Allocator& operator=(Allocator&&) = delete;`.
    // In Rust, this is represented by not implementing the `Copy` or `Clone` traits,
    // and specifically not providing a move-assignment equivalent if the type
    // is intended to be pinned or non-assignable.
    // Since it is explicitly deleted in C++, we do not provide a functional implementation.
}
