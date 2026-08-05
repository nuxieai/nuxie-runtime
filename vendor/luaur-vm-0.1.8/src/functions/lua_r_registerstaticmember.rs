use crate::macros::lua_c_barrier::luaC_barrier;
use crate::macros::setnvalue::setnvalue;
use crate::macros::setobj_2_class::setobj2class;
use crate::records::lua_state::lua_State;
use crate::records::luau_class::LuauClass;
use crate::records::t_string::TString;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_r_registerstaticmember(
    L: *mut lua_State,
    classobject: *mut LuauClass,
    member_name: *mut TString,
    val: *const TValue,
    offset: u32,
    static_member_offset: u32,
) {
    let destination = (*classobject).staticmembers.add(static_member_offset as usize);
    setobj2class!(L, destination, val);
    luaC_barrier!(L, classobject, destination as *const TValue);
    *(*classobject).offsettomember.add(offset as usize) = member_name;
    let offset_val = crate::functions::lua_h_setstr::lua_h_setstr(
        L,
        (*classobject).memberstooffset,
        member_name,
    );
    setnvalue!(offset_val, offset as f64);
    luaC_barrier!(L, (*classobject).memberstooffset, offset_val as *const TValue);
}
