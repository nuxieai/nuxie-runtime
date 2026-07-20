#[allow(non_snake_case)]
macro_rules! freegcolink {
    ($block:expr) => {
        unsafe {
            let ptr = ($block as *mut core::ffi::c_char).add(crate::records::lmem::kGCOLinkOffset);
            &mut *(ptr as *mut *mut core::ffi::c_void)
        }
    };
}

pub(crate) use freegcolink;
