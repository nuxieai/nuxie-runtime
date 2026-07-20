use crate::enums::lua_type::lua_Type;
use crate::macros::lua_c_init::luaC_init;
use crate::macros::lua_m_newarray::luaM_newarray;
use crate::macros::setclvalue::setclvalue;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::ttisnil::ttisnil;
use crate::records::closure::CClosure;
use crate::records::lua_state::lua_State;
use crate::records::lua_table::LuaTable;
use crate::records::luau_class::LuauClass;
use crate::records::t_string::TString;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::tms::TMS;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn lua_r_newclass(
    L: *mut lua_State,
    name: *mut TString,
    memberstooffset: *mut LuaTable,
    offsettomember: *mut *mut TString,
    numberofinstancemembers: i32,
    numberofstaticmembers: i32,
) -> *mut LuauClass {
    let global = (*L).global;
    LUAU_ASSERT!((*global).GCthreshold == usize::MAX);

    let classobject = crate::functions::lua_m_newgco::luaM_newgco_(
        L,
        core::mem::size_of::<LuauClass>(),
        (*L).activememcat,
    ) as *mut LuauClass;
    luaC_init!(L, classobject, lua_Type::LUA_TCLASS as core::ffi::c_int);

    (*classobject).name = name;

    (*classobject).staticmembers =
        luaM_newarray!(L, numberofstaticmembers, TValue, (*classobject).memcat);
    for i in 0..numberofstaticmembers {
        setnilvalue!((*classobject).staticmembers.add(i as usize));
    }

    (*classobject).memberstooffset = memberstooffset;
    (*classobject).offsettomember = offsettomember;

    (*classobject).metatable = crate::functions::lua_h_new::lua_h_new(L, 0, 1);
    let constructor = crate::functions::lua_f_new_cclosure::lua_f_new_cclosure(L, 0, (*L).gt);
    let constructor_c = core::ptr::addr_of_mut!((*constructor).inner.c) as *mut CClosure;
    (*constructor_c).f = Some(crate::functions::lua_r_createobject::lua_r_createobject);
    (*constructor_c).debugname = c"luaR_createobject".as_ptr();
    (*constructor_c).cont = None;
    let dest = crate::functions::lua_h_setstr::lua_h_setstr(
        L,
        (*classobject).metatable,
        (*global).tmname[TMS::TM_CALL as usize],
    );
    LUAU_ASSERT!(ttisnil!(dest));
    setclvalue!(L, dest, constructor);
    (*(*classobject).metatable).readonly = 1;
    (*classobject).instancemetatable = core::ptr::null_mut();

    (*classobject).numberofinstancemembers = numberofinstancemembers;
    (*classobject).numberofallmembers = numberofinstancemembers + numberofstaticmembers;

    classobject
}
