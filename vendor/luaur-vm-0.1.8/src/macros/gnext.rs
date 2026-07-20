// VM/src/ltable.h — #define gnext(n) ((n)->key.next)
// TKey packs tt+next into tt_next; the record exposes a next() accessor.
#[allow(non_snake_case)]
#[macro_export]
macro_rules! gnext {
    ($n:expr) => {
        (*$n).key.next()
    };
}

pub use gnext;
