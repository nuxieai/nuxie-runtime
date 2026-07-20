#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_SETJMP {
    ($buf:expr) => {
        unsafe { libc::_setjmp($buf) }
    };
}

pub use LUAU_SETJMP;
