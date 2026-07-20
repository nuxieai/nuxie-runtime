use crate::enums::lua_type::lua_Type;
use crate::functions::lua_m_visitgco::lua_m_visitgco;
use crate::functions::validategco::validategco;
use crate::functions::validategraylist::validategraylist;
use crate::macros::checkliveness::checkliveness;
use crate::macros::isblack::isblack;
use crate::macros::isdead::isdead;
use crate::macros::obj_2_gco::obj2gco;
use crate::macros::upisopen::upisopen;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use crate::records::lua_page::lua_Page;
use crate::records::lua_state::lua_State;
use crate::type_aliases::up_val::UpVal;
use core::ffi::c_void;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn lua_c_validate(L: *mut lua_State) {
    let g: *mut global_State = (*L).global;

    // The obj2gco! macro relies on ttype!, which expects a .tt() method.
    // Since the Rust records for lua_State, LuaTable, and UpVal use a field hdr.tt instead of a method,
    // we must cast these pointers to *mut GCObject (which is what obj2gco! effectively does via cast_to!)
    // to bypass the ttype! check on the specific record types.
    LUAU_ASSERT!(!isdead!(g, (*g).mainthread as *mut GCObject));
    checkliveness!(g, &(*g).registry);

    for i in 0..(crate::enums::lua_type::LUA_T_COUNT as i32) {
        let mt = (*g).mt[i as usize];
        if !mt.is_null() {
            LUAU_ASSERT!(!isdead!(g, mt as *mut GCObject));
        }
    }

    validategraylist(g, (*g).weak as *mut GCObject);
    validategraylist(g, (*g).gray as *mut GCObject);
    validategraylist(g, (*g).grayagain as *mut GCObject);

    validategco(
        L as *mut c_void,
        core::ptr::null_mut::<lua_Page>(),
        (*g).mainthread as *mut GCObject,
    );

    lua_m_visitgco(L, L as *mut c_void, validategco as *mut c_void);

    let mut uv: *mut UpVal = (*g).uvhead.u.open.next;
    while uv != &mut (*g).uvhead {
        LUAU_ASSERT!((*uv).hdr.tt == lua_Type::LUA_TUPVAL as u8);
        LUAU_ASSERT!(upisopen!(uv));
        LUAU_ASSERT!(
            (*(*uv).u.open.next).u.open.prev == uv && (*(*uv).u.open.prev).u.open.next == uv
        );
        // open upvalues are never black
        LUAU_ASSERT!(!isblack!(uv as *mut GCObject));
        uv = (*uv).u.open.next;
    }
}
