#[macro_export]
#[allow(non_snake_case)]
macro_rules! api_update_top {
    ($L:expr, $p:expr) => {{
        let L_ptr = $L;
        let p_val = $p;
        crate::macros::api_check::api_check!(
            L_ptr,
            p_val >= (*L_ptr).base && p_val <= (*(*L_ptr).ci).top
        );
        (*L_ptr).top = p_val;
    }};
}

pub use api_update_top;
