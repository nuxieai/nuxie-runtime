use crate::enums::lua_type::lua_Type;
use crate::functions::lua_h_getstr::lua_h_getstr;
use crate::functions::lua_h_new::lua_h_new;
use crate::functions::lua_h_setstr::lua_h_setstr;
use crate::functions::lua_s_newlstr::lua_s_newlstr;
use crate::macros::lua_c_barrier::luaC_barrier;
use crate::macros::lua_c_objbarrier::luaC_objbarrier;
use crate::macros::nvalue::nvalue;
use crate::macros::setobj_2_class::setobj2class;
use crate::macros::setobj_2_t::setobj2t;
use crate::macros::ttisfunction::ttisfunction;
use crate::macros::ttisnumber::ttisnumber;
use crate::records::global_state::global_State;
use crate::records::lua_state::lua_State;
use crate::records::luau_class::LuauClass;
use crate::records::t_string::TString;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::tms::TMS;
use luaur_common::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn lua_r_addclassmember(
    L: *mut lua_State,
    classobject: *mut LuauClass,
    name: *mut TString,
    value: *mut TValue,
) {
    LUAU_ASSERT!(!(*classobject).staticmembers.is_null());

    let offset = lua_h_getstr((*classobject).memberstooffset, name);
    LUAU_ASSERT!(ttisnumber!(offset));
    let offsetint = nvalue!(offset) as i32;
    LUAU_ASSERT!(
        offsetint >= (*classobject).numberofinstancemembers
            && offsetint < (*classobject).numberofallmembers
    );
    LUAU_ASSERT!(
        ttisfunction!(value) && (*(*value).value.gc).gch.tt == lua_Type::LUA_TFUNCTION as u8
    );
    setobj2class!(
        L,
        (*classobject)
            .staticmembers
            .add((offsetint - (*classobject).numberofinstancemembers) as usize),
        value
    );
    luaC_barrier!(L, classobject, value);

    // Only metamethods in the parser's allowlist are supported (see ALLOWED_METAMETHODS in Parser.cpp)
    let isMetamethod = (name == lua_s_newlstr(L, b"__tostring\0" as *const _ as *const _, 10));
    let mut isMetamethod = isMetamethod;
    let g: *mut global_State = (*L).global;
    for i in 0..TMS::TM_N as usize {
        if isMetamethod {
            break;
        }
        isMetamethod = (name == (*g).tmname[i]);
    }

    if isMetamethod {
        if (*classobject).instancemetatable.is_null() {
            (*classobject).instancemetatable = lua_h_new(L, 0, 1);
            luaC_objbarrier!(L, classobject, (*classobject).instancemetatable);
        }
        let dest = lua_h_setstr(L, (*classobject).instancemetatable, name);
        setobj2t!(L, dest, value);
        luaC_barrier!(L, (*classobject).instancemetatable, value);
    }
}
