use crate::enums::lua_status::lua_Status;
use crate::functions::arrayornewkey::arrayornewkey;
use crate::functions::lua_d_throw_ldo::luaD_throw;
use crate::functions::lua_g_pusherror::lua_g_pusherror;
use crate::functions::lua_m_free::luaM_free_;
use crate::functions::newkey::newkey;
use crate::functions::setarrayvector::setarrayvector;
use crate::functions::setnodevector::setnodevector;
use crate::macros::cast_num::cast_num;
use crate::macros::dummynode::dummynode;
use crate::macros::getnodekey::getnodekey;
use crate::macros::gkey::gval;
use crate::macros::setnvalue::setnvalue;
use crate::macros::setobjt_2_t::setobjt2t;
use crate::macros::ttisnil::ttisnil;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;
use core::ffi::c_int;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

const MAXSIZE: c_int = 1 << 26;

unsafe fn runerror(l: *mut lua_State, msg: *const core::ffi::c_char) -> ! {
    lua_g_pusherror(l, msg);
    luaD_throw(l, lua_Status::LUA_ERRRUN as c_int);
}

#[allow(non_snake_case)]
pub unsafe fn resize(l: *mut lua_State, t: *mut LuaTable, nasize: c_int, nhsize: c_int) {
    if nasize > MAXSIZE || nhsize > MAXSIZE {
        runerror(l, c"table overflow".as_ptr());
    }

    let oldasize = (*t).sizearray;
    let oldhsize = (*t).lsizenode;
    let nold = (*t).node;

    if nasize > oldasize {
        setarrayvector(l, t, nasize);
    }

    setnodevector(l, t, nhsize);
    let nnew = (*t).node;

    if nasize < oldasize {
        (*t).sizearray = nasize;

        let mut i = nasize;
        while i < oldasize {
            let e = (*t).array.add(i as usize);
            if !ttisnil!(e) {
                let mut ok = TValue::default();
                setnvalue!(core::ptr::addr_of_mut!(ok), cast_num!(i + 1));
                setobjt2t!(l, newkey(l, t, core::ptr::addr_of!(ok)), e);
            }
            i += 1;
        }

        let newarray = crate::functions::lua_m_realloc::lua_m_realloc_(
            l,
            (*t).array as *mut core::ffi::c_void,
            oldasize as usize * core::mem::size_of::<TValue>(),
            nasize as usize * core::mem::size_of::<TValue>(),
            (*t).memcat,
        ) as *mut TValue;
        (*t).array = newarray;
    }

    let anew = (*t).array;

    let oldhsize_slots = 1i32 << oldhsize;
    let mut i = oldhsize_slots - 1;
    while i >= 0 {
        let old = nold.add(i as usize);
        if !ttisnil!(gval!(old)) {
            let mut ok = TValue::default();
            getnodekey!(l, core::ptr::addr_of_mut!(ok), old);
            setobjt2t!(l, arrayornewkey(l, t, core::ptr::addr_of!(ok)), gval!(old));
        }

        if i == 0 {
            break;
        }
        i -= 1;
    }

    LUAU_ASSERT!(nnew == (*t).node);
    LUAU_ASSERT!(anew == (*t).array);

    if nold != dummynode as *mut LuaNode {
        luaM_free_(
            l,
            nold as *mut core::ffi::c_void,
            oldhsize_slots as usize * core::mem::size_of::<LuaNode>(),
            (*t).memcat,
        );
    }
}
