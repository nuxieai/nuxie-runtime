#[allow(non_snake_case)]
macro_rules! metadata {
    ($block:expr) => {
        *($block as *mut *mut core::ffi::c_void)
    };
}

pub(crate) use metadata;

#[allow(non_snake_case)]
macro_rules! freegcolink {
    ($block:expr) => {
        *(($block as *mut core::ffi::c_char).offset(crate::conf::kGCOLinkOffset as isize)
            as *mut *mut core::ffi::c_void)
    };
}

pub(crate) use freegcolink;
