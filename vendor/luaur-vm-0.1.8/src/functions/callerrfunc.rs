//! Node: `cxx:Function:Luau.VM:VM/src/ldo.cpp:718:callerrfunc`
//! Source: `VM/src/ldo.cpp` (ldo.cpp:718-727, hand-ported)

use crate::functions::lua_d_callny::lua_d_callny;
use crate::macros::incr_top::incr_top;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

pub unsafe fn callerrfunc(l: *mut lua_State, ud: *mut core::ffi::c_void) {
    let errfunc = ud as StkId;

    setobj_2_s!(l, (*l).top, (*l).top.offset(-1));
    setobj_2_s!(l, (*l).top.offset(-1), errfunc);
    incr_top!(l);

    lua_d_callny(l, (*l).top.offset(-2), 1);
}
