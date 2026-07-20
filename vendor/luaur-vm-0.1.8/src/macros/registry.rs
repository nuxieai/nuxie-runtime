#[allow(non_snake_case)]
#[macro_export]
macro_rules! registry {
    ($L:expr) => {
        &(*(*$L).global).registry
    };
}

pub use registry;
