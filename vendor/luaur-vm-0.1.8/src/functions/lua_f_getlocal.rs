use core::ffi::c_int;
use core::ptr;

use crate::type_aliases::loc_var::LocVar;
use crate::type_aliases::proto::Proto;

#[allow(non_snake_case)]
pub fn luaF_getlocal(func: *const Proto, mut local_number: c_int, pc: c_int) -> *const LocVar {
    if func.is_null() {
        return ptr::null();
    }

    unsafe {
        for i in 0..(*func).sizelocvars {
            let loc = &*(*func).locvars.add(i as usize);
            if pc >= loc.startpc && pc < loc.endpc {
                local_number -= 1;
                if local_number == 0 {
                    return loc as *const LocVar;
                }
            }
        }
    }

    ptr::null()
}
