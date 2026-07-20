#[macro_export]
#[allow(non_snake_case)]
macro_rules! api_check {
    ($l:expr, $e:expr) => {
        luaur_common::LUAU_ASSERT!($e);
    };
}

pub use api_check;
