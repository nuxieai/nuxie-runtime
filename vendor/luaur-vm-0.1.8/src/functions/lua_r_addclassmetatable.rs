use crate::macros::setclvalue::setclvalue;
use crate::macros::ttisnil::ttisnil;
use crate::records::closure::CClosure;
use crate::macros::lua_s_new::luaS_new;
use crate::records::lua_state::lua_State;
use crate::records::luau_class::LuauClass;
use crate::type_aliases::tms::TMS;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn lua_r_addclassmetatable(L: *mut lua_State, classobject: *mut LuauClass) {
    (*classobject).metatable = crate::functions::lua_h_new::lua_h_new(L, 0, 1);
    let constructor = crate::functions::lua_f_new_cclosure::lua_f_new_cclosure(L, 0, (*L).gt);
    let constructor_c = core::ptr::addr_of_mut!((*constructor).inner.c) as *mut CClosure;
    (*constructor_c).f = Some(crate::functions::lua_r_createobject::lua_r_createobject);
    if luaur_common::FFlag::LuauManagedDebugNames.get() {
        (*constructor_c).debugname = luaS_new(L, c"luaR_createobject".as_ptr());
    } else {
        (*constructor_c).debugname_DEPRECATED = c"luaR_createobject".as_ptr();
    }
    (*constructor_c).cont = None;
    let dest = crate::functions::lua_h_setstr::lua_h_setstr(
        L,
        (*classobject).metatable,
        (*(*L).global).tmname[TMS::TM_CALL as usize],
    );
    LUAU_ASSERT!(ttisnil!(dest));
    setclvalue!(L, dest, constructor);
    (*(*classobject).metatable).readonly = 1;
}
