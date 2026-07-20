#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_NORETURN {
    ($item:item) => {
        #[noreturn]
        $item
    };
}

pub use LUAU_NORETURN;
