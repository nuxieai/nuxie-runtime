#[allow(non_snake_case)]
#[macro_export]
macro_rules! gkey {
    ($n:expr) => {
        core::ptr::addr_of_mut!((*$n).key)
    };
}

pub use gkey;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! gval {
    ($n:expr) => {
        core::ptr::addr_of_mut!((*$n).val)
    };
}

pub use gval;
