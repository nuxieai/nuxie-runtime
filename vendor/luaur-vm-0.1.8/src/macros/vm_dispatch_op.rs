#[allow(non_snake_case)]
#[macro_export]
macro_rules! VM_DISPATCH_OP {
    ($op:ident) => {
        // In C++, this expands to &&CASE_##op (a GCC/Clang label address).
        // In Rust, we use match arms or a similar dispatch mechanism.
        // To avoid "unknown prefix" errors in Rust 2021+, we use concat_idents-like behavior
        // or simply rely on the downstream match arm naming convention.
        // Since Rust doesn't have a stable concat_idents!, we emit the identifier directly
        // via a token-pasting-friendly pattern if needed, but for Luau VM dispatch,
        // this usually refers to a variant or a label in a manual dispatch table.
        $op
    };
}

pub use VM_DISPATCH_OP;
