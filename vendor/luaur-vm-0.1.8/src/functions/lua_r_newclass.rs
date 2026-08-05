use crate::macros::lua_m_newarray::luaM_newarray;
use crate::macros::setnilvalue::setnilvalue;
use crate::records::lua_state::lua_State;
use crate::records::lua_table::LuaTable;
use crate::records::luau_class::LuauClass;
use crate::records::t_string::TString;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn lua_r_newclass(
    L: *mut lua_State,
    name: *mut TString,
    memberstooffset: *mut LuaTable,
    offsettomember: *mut *mut TString,
    numberofinstancemembers: u32,
    numberofstaticmembers: u32,
) -> *mut LuauClass {
    let global = (*L).global;
    LUAU_ASSERT!((*global).GCthreshold == usize::MAX);

    let classobject = crate::functions::lua_r_newblankclass::lua_r_newblankclass(L, name);

    (*classobject).staticmembers =
        luaM_newarray!(L, numberofstaticmembers, TValue, (*classobject).memcat);
    for i in 0..numberofstaticmembers {
        setnilvalue!((*classobject).staticmembers.add(i as usize));
    }

    (*classobject).memberstooffset = memberstooffset;
    (*classobject).offsettomember = offsettomember;

    (*classobject).numberofinstancemembers = numberofinstancemembers;
    (*classobject).numberofallmembers = numberofinstancemembers + numberofstaticmembers;

    crate::functions::lua_r_addclassmetatable::lua_r_addclassmetatable(L, classobject);
    (*classobject).instancemetatable = core::ptr::null_mut();

    classobject
}
