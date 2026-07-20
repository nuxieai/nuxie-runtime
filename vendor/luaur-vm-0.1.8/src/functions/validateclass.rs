use crate::functions::validateobjref::validateobjref;
use crate::functions::validateref::validateref;
use crate::macros::obj_2_gco::obj2gco;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use crate::records::luau_class::LuauClass;

#[allow(non_snake_case)]
pub(crate) unsafe fn validateclass(g: *mut global_State, lco: *mut LuauClass) {
    let obj = obj2gco!(lco as *mut LuauClass);
    validateobjref(
        g,
        obj,
        obj2gco!((*lco).name as *mut crate::records::t_string::TString),
    );
    validateobjref(
        g,
        obj,
        obj2gco!((*lco).memberstooffset as *mut crate::records::lua_table::LuaTable),
    );

    for i in 0..(*lco).numberofallmembers {
        validateobjref(
            g,
            obj,
            obj2gco!(
                *(*lco).offsettomember.add(i as usize) as *mut crate::records::t_string::TString
            ),
        );
        if i >= (*lco).numberofinstancemembers {
            validateref(
                g,
                obj,
                (*lco)
                    .staticmembers
                    .add((i - (*lco).numberofinstancemembers) as usize),
            );
        }
    }

    validateobjref(
        g,
        obj,
        obj2gco!((*lco).metatable as *mut crate::records::lua_table::LuaTable),
    );
    if !(*lco).instancemetatable.is_null() {
        validateobjref(
            g,
            obj,
            obj2gco!((*lco).instancemetatable as *mut crate::records::lua_table::LuaTable),
        );
    }
}
