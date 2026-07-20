use crate::functions::dumpref::dumpref;
use crate::macros::gcvalue::gcvalue;
use crate::macros::iscollectable::iscollectable;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumprefs(f: *mut core::ffi::c_void, data: *mut TValue, size: usize) {
    let mut first = true;

    for i in 0..size {
        let val_ptr = data.add(i);
        if iscollectable!(val_ptr) {
            if !first {
                extern "C" {
                    fn fputc(
                        c: core::ffi::c_int,
                        stream: *mut core::ffi::c_void,
                    ) -> core::ffi::c_int;
                }
                fputc(',' as core::ffi::c_int, f);
            }
            first = false;

            dumpref(f, gcvalue!(val_ptr));
        }
    }
}
