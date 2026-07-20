#[macro_export]
#[allow(non_snake_case)]
macro_rules! api_incr_top {
    ($L:expr) => {
        unsafe {
            crate::macros::api_check::api_check!($L, (*$L).top < (*(*$L).ci).top);
            (*$L).top = (*$L).top.add(1);
        }
    };
}

pub use api_incr_top;
