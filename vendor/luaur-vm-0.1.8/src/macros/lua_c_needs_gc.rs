#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaC_needsGC {
    ($L:expr) => {
        (*(*$L).global).totalbytes >= (*(*$L).global).GCthreshold
    };
}

pub use luaC_needsGC;
