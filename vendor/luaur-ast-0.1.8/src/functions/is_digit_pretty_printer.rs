#[allow(non_snake_case)]
pub fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

// Pinned overload name advertised by the dependency cards.
#[allow(unused_imports, non_snake_case)]
pub use is_digit as is_digit_mut;
