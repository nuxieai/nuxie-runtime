//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:805:markroot`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:805-837, hand-ported)

use crate::functions::markmt::markmt;
use crate::macros::gc_spropagate::GCSpropagate;
use crate::macros::markobject::markobject;
use crate::macros::markvalue::markvalue;
use crate::macros::utag_internal_limit::UTAG_INTERNAL_LIMIT;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

// mark root set
#[allow(non_snake_case)]
pub(crate) unsafe fn markroot(l: *mut lua_State) {
    let g = (*l).global;
    (*g).gray = core::ptr::null_mut();
    (*g).grayagain = core::ptr::null_mut();
    (*g).weak = core::ptr::null_mut();
    markobject!(g, (*g).mainthread);
    // make global table be traversed before main stack
    markobject!(g, (*(*g).mainthread).gt);
    // registry(L) — &L->global->registry
    markvalue!(g, core::ptr::addr_of_mut!((*g).registry));

    if luaur_common::FFlag::LuauUdataDirectAccess6.get() {
        for i in 0..UTAG_INTERNAL_LIMIT as usize {
            let udatadirect = core::ptr::addr_of_mut!((*(*l).global).udatadirect[i]);

            markvalue!(
                g,
                core::ptr::addr_of_mut!((*udatadirect).indextm) as *mut TValue
            );
            markvalue!(
                g,
                core::ptr::addr_of_mut!((*udatadirect).newindextm) as *mut TValue
            );
            markvalue!(
                g,
                core::ptr::addr_of_mut!((*udatadirect).namecalltm) as *mut TValue
            );
        }
    }

    if luaur_common::FFlag::LuauDirectFieldGet.get() {
        for i in 0..UTAG_INTERNAL_LIMIT as usize {
            if !(*g).udatadirectfields[i].is_null() {
                markobject!(g, (*g).udatadirectfields[i]);
            }
        }
    }

    markmt(g);
    (*g).gcstate = GCSpropagate as u8;
}
