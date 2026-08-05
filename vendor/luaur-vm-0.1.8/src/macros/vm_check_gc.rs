#[allow(non_snake_case)]
#[macro_export]
macro_rules! VM_CHECK_GC {
    ($L:expr, $pc:expr, $base:expr) => {{
        if $crate::macros::lua_c_needs_gc::luaC_needsGC!($L) {
            unsafe {
                (*(*$L).ci).context.savedpc = $pc;
                $crate::functions::lua_c_step::luaC_step($L, true);
                $base = (*$L).base;
            }
        }
    }};
}

pub use VM_CHECK_GC;
