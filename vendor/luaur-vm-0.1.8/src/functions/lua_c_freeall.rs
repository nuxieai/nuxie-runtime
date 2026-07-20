use crate::functions::deletegco::deletegco;
use crate::functions::lua_m_visitgco::lua_m_visitgco;
use crate::records::global_state::global_State;
use crate::records::lua_state::lua_State;
use crate::records::t_string::TString;
use core::ffi::c_void;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaC_freeall(L: *mut lua_State) {
    let g: *mut global_State = (*L).global;

    LUAU_ASSERT!(L == (*g).mainthread);

    lua_m_visitgco(L, L as *mut c_void, deletegco as *mut c_void);

    for i in 0..(*g).strt.size {
        // free all string lists
        let bucket: *mut TString = *((*g).strt.hash.add(i as usize));
        LUAU_ASSERT!(bucket.is_null());
    }

    LUAU_ASSERT!((*(*L).global).strt.nuse == 0);
}
