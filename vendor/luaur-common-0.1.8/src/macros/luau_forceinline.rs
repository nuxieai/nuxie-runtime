#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_FORCEINLINE {
    ($item:item) => {
        #[inline(always)]
        $item
    };
}

pub use LUAU_FORCEINLINE;
