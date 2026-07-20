use core::ffi::c_int;

use crate::records::loc_var::LocVar;
use crate::records::proto::Proto;

#[allow(non_snake_case)]
pub unsafe fn luaF_findlocal(f: *const Proto, local_reg: c_int, pc: c_int) -> *const LocVar {
    let mut i: c_int = 0;
    let sizelocvars = (*f).sizelocvars;
    let locvars = (*f).locvars;

    while i < sizelocvars {
        let locvar = &*locvars.add(i as usize);
        if local_reg == locvar.reg as c_int && pc >= locvar.startpc && pc < locvar.endpc {
            return locvars.add(i as usize) as *const LocVar;
        }
        i += 1;
    }

    core::ptr::null()
}
