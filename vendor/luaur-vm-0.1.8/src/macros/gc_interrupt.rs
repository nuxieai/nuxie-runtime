#[allow(non_snake_case)]
#[macro_export]
macro_rules! GC_INTERRUPT {
    ($L:expr, $state:expr) => {
        unsafe {
            let g = &*(*$L).global;
            let interrupt = g.cb.interrupt;
            if luaur_common::LUAU_UNLIKELY(interrupt.is_some()) {
                if let Some(interrupt_fn) = interrupt {
                    interrupt_fn($L, $state);
                }
            }
        }
    };
}

pub use GC_INTERRUPT;
