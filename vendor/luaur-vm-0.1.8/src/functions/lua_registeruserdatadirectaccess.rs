use core::ffi::c_int;

use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::macros::api_check::api_check;
use crate::macros::fasttm::fasttm;
use crate::macros::lua_utag_limit::LUA_UTAG_LIMIT;
use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_userdata_direct_access::lua_UserdataDirectAccess;
use crate::type_aliases::lua_userdata_direct_namecall::lua_UserdataDirectNamecall;
use crate::type_aliases::tms::TMS;

#[allow(non_snake_case)]
pub unsafe fn lua_registeruserdatadirectaccess(
    L: *mut lua_State,
    tag: c_int,
    get: lua_UserdataDirectAccess,
    set: lua_UserdataDirectAccess,
    namecall: lua_UserdataDirectNamecall,
) -> c_int {
    api_check!(L, (tag as u32) < LUA_UTAG_LIMIT as u32);
    lua_c_threadbarrier_lapi(L);

    let h = (*(*L).global).udatamt[tag as usize];
    if !h.is_null() {
        let udatadirect = &mut (*(*L).global).udatadirect[tag as usize];

        let indextm = fasttm(L, h, TMS::TM_INDEX as c_int);
        if !indextm.is_null() {
            udatadirect.indextm = *indextm;
            udatadirect.index = get;
        }

        let newindextm = fasttm(L, h, TMS::TM_NEWINDEX as c_int);
        if !newindextm.is_null() {
            udatadirect.newindextm = *newindextm;
            udatadirect.newindex = set;
        }

        let namecalltm = fasttm(L, h, TMS::TM_NAMECALL as c_int);
        if !namecalltm.is_null() {
            udatadirect.namecalltm = *namecalltm;
            udatadirect.namecall = namecall;
        }

        return 1;
    }

    0
}
