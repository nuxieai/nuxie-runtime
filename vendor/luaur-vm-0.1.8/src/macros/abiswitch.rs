#[allow(non_snake_case)]
macro_rules! ABISWITCH {
    ($x64:expr, $ms32:expr, $gcc32:expr) => {
        if core::mem::size_of::<*mut core::ffi::c_void>() == 8 {
            $x64
        } else {
            $gcc32
        }
    };
}

pub(crate) use ABISWITCH;
