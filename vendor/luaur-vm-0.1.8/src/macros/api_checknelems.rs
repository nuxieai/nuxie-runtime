#[macro_export]
#[allow(non_snake_case)]
macro_rules! api_checknelems {
    ($L:expr, $n:expr) => {
        crate::macros::api_check::api_check!(
            $L,
            ($n) as isize <= unsafe { (*$L).top.offset_from((*$L).base) }
        );
    };
}

pub use api_checknelems;
