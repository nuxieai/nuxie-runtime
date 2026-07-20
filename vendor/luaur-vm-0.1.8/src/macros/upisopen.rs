// VM/src/lfunc.h:14 — #define upisopen(up) ((up)->v != &(up)->u.value)
#[allow(non_snake_case)]
#[macro_export]
macro_rules! upisopen {
    ($up:expr) => {
        (*$up).v
            != core::ptr::addr_of_mut!((*$up).u.value) as *mut $crate::type_aliases::t_value::TValue
    };
}

pub use upisopen;
