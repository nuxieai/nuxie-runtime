#[macro_export]
macro_rules! vm_protect {
    ($L:expr, $pc:expr, $base:expr, $x:expr) => {
        unsafe {
            (*(*$L).ci).context.savedpc = $pc;
            {
                $x;
            };
            $base = (*$L).base;
        }
    };
}

pub use vm_protect;
