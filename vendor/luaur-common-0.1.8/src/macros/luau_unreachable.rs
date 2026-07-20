#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_UNREACHABLE {
    () => {
        unsafe {
            core::hint::unreachable_unchecked();
        }
    };
}

pub use LUAU_UNREACHABLE;
