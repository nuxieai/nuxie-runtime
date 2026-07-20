use crate::functions::lua_c_barrierback::lua_c_barrierback;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::blackbit::BLACKBIT;
use crate::macros::setthvalue::setthvalue;
use crate::records::gc_object::GCObject;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn lua_pushthread(l: *mut lua_State) -> i32 {
    // Inline luaC_threadbarrier(l) to avoid buggy macros in the crate
    let marked = (*l).hdr.marked as i32;
    if (marked & (1 << BLACKBIT)) != 0 {
        lua_c_barrierback(
            l,
            l as *mut GCObject,
            &mut (*l).gclist as *mut *mut GCObject,
        );
    }

    setthvalue!(l, (*l).top, l);
    api_incr_top!(l);
    ((*(*l).global).mainthread == l) as i32
}
