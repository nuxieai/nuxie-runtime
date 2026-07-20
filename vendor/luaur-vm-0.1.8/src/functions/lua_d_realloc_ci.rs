use crate::functions::lua_m_realloc::lua_m_realloc_;
use crate::records::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn luaD_reallocCI(l: *mut lua_State, newsize: c_int) {
    let oldci = (*l).base_ci;
    let oldoffset = (*l).ci.offset_from(oldci);

    (*l).base_ci = lua_m_realloc_(
        l,
        (*l).base_ci as *mut core::ffi::c_void,
        (*l).size_ci as usize * core::mem::size_of::<CallInfo>(),
        newsize as usize * core::mem::size_of::<CallInfo>(),
        (*l).hdr.memcat,
    ) as *mut CallInfo;

    (*l).size_ci = newsize;
    (*l).ci = (*l).base_ci.offset(oldoffset);
    (*l).end_ci = (*l).base_ci.add(((*l).size_ci - 1) as usize);
}

#[allow(unused_imports)]
pub use luaD_reallocCI as lua_d_realloc_ci;
