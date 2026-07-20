use crate::functions::lua_m_freegco::luaM_freegco_;
use crate::functions::unlinkstr::unlinkstr;
use crate::macros::sizestring::sizestring;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use crate::records::t_string::TString;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaS_free(l: *mut lua_State, ts: *mut TString, page: *mut lua_Page) {
    if unlinkstr(l, ts) {
        (*(*l).global).strt.nuse = (*(*l).global).strt.nuse.wrapping_sub(1);
    } else {
        LUAU_ASSERT!((*ts).next.is_null());
    }

    luaM_freegco_(
        l,
        ts as *mut GCObject,
        sizestring((*ts).len as usize),
        (*ts).hdr.memcat,
        page,
    );
}

#[allow(unused_imports)]
pub use luaS_free as lua_s_free;
