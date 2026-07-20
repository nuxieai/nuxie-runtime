#[allow(non_snake_case)]
macro_rules! LUAU_LONGJMP {
    ($buf:expr, $code:expr) => {
        unsafe { core::ffi::c_int::from(libc::_longjmp($buf, $code)) }
    };
}

pub(crate) use LUAU_LONGJMP;
