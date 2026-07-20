#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_NOINLINE {
    ($item:item) => {
        #[inline(never)]
        $item
    };
}

pub use LUAU_NOINLINE;
