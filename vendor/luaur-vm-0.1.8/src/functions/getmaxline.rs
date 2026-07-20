use crate::functions::lua_g_getline::luaG_getline;
use crate::records::proto::Proto;

pub(crate) unsafe fn getmaxline(p: *mut Proto) -> core::ffi::c_int {
    let mut result: core::ffi::c_int = -1;

    for i in 0..(*p).sizecode {
        let line = luaG_getline(p, i);
        result = if result < line { line } else { result };
    }

    for i in 0..(*p).sizep {
        let psize = getmaxline(*(*p).p.add(i as usize));
        result = if result < psize { psize } else { result };
    }

    result
}
