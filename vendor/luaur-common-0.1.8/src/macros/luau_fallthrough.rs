#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_FALLTHROUGH {
    () => {
        // Rust uses the `break` or implicit fallthrough is not allowed in match arms.
        // In C++, this macro is used to suppress warnings for intentional fallthrough in switch statements.
        // In Rust, intentional fallthrough is expressed by combining match patterns (e.g., `A | B => ...`).
        // This macro is provided for source compatibility but is a no-op as Rust does not support C-style switch fallthrough.
    };
}

pub use LUAU_FALLTHROUGH;
