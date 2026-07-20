//! Node: `cxx:Function:Luau.VM:VM/src/lfunc.cpp:156:lua_f_closeupval`
//! Source: `VM/src/lfunc.cpp` (lfunc.cpp:156-169, hand-ported)

use crate::functions::lua_c_upvalclosed::luaC_upvalclosed;
use crate::macros::setobj::setobj;
use crate::records::up_val::UpVal;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaF_closeupval(l: *mut lua_State, uv: *mut UpVal, dead: bool) {
    // unlink value from all lists *before* closing it since value storage overlaps
    LUAU_ASSERT!((*(*uv).u.open.next).u.open.prev == uv && (*(*uv).u.open.prev).u.open.next == uv);
    (*(*uv).u.open.next).u.open.prev = (*uv).u.open.prev;
    (*(*uv).u.open.prev).u.open.next = (*uv).u.open.next;

    if dead {
        return;
    }

    let value = core::ptr::addr_of_mut!((*uv).u.value) as *mut TValue;
    setobj!(l, value, (*uv).v);
    (*uv).v = value;
    luaC_upvalclosed(l, uv);
}

#[allow(unused_imports)]
pub use luaF_closeupval as lua_f_closeupval;
