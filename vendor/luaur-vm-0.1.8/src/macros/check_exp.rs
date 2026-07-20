#[allow(non_snake_case)]
#[macro_export]
macro_rules! check_exp {
    ($c:expr, $e:expr) => {{
        luaur_common::LUAU_ASSERT!($c);
        $e
    }};
}

pub use check_exp;
