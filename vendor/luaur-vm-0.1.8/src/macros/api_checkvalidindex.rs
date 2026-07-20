use crate::macros::api_check::api_check;

#[macro_export]
#[allow(non_snake_case)]
macro_rules! api_checkvalidindex {
    ($L:expr, $i:expr) => {
        crate::macros::api_check::api_check!($L, $i != crate::records::lobject::luaO_nilobject);
    };
}

pub use api_checkvalidindex;
