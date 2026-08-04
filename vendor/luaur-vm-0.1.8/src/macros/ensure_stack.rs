#[macro_export]
#[allow(non_snake_case)]
macro_rules! ensure_stack {
    ($L:expr, $size:expr) => {
        $crate::macros::ensure_stack_impl::ensure_stack_impl!($L, $L, $size)
    };
}

pub use ensure_stack;
