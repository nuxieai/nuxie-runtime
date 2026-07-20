use crate::functions::lua_c_barrierback::lua_c_barrierback;
use crate::functions::luau_execute::luau_execute;
use crate::functions::luau_precall::luau_precall;
use crate::macros::isblack::isblack;
use crate::macros::lua_callinfo_return::LUA_CALLINFO_RETURN;
use crate::macros::pcrlua::PCRLUA;
use crate::macros::scheduled_reentry::SCHEDULED_REENTRY;
use crate::records::gc_object::GCObject;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn performcall(
    l: *mut lua_State,
    func: StkId,
    nresults: core::ffi::c_int,
    preparereentry: bool,
) {
    if luau_precall(l, func, nresults) == PCRLUA {
        (*(*l).ci).flags |= LUA_CALLINFO_RETURN as u32;

        let oldactive = (*l).isactive;
        (*l).isactive = true;

        let o = l as *mut GCObject;
        if isblack!(o) {
            lua_c_barrierback(l, o, core::ptr::addr_of_mut!((*l).gclist));
        }

        if preparereentry {
            (*l).status = SCHEDULED_REENTRY as u8;
        } else {
            luau_execute(l);
        }

        if !oldactive {
            (*l).isactive = false;
        }
    }
}
