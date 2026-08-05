use crate::macros::getstr::getstr;
use crate::macros::lua_c_barrier::luaC_barrier;
use crate::macros::lua_c_objbarrier::luaC_objbarrier;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::macros::lua_m_newarray::luaM_newarray;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnil::ttisnil;
use crate::records::lua_state::lua_State;
use crate::records::luau_class::LuauClass;
use crate::records::t_string::TString;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

unsafe fn string_value(value: *mut TString) -> alloc::string::String {
    core::ffi::CStr::from_ptr(getstr(value))
        .to_string_lossy()
        .into_owned()
}

#[allow(non_snake_case)]
pub unsafe fn lua_r_inheritclass(
    L: *mut lua_State,
    child: *const LuauClass,
    parent: *const LuauClass,
) -> *mut LuauClass {
    for idx in 0..(*parent).numberofinstancemembers {
        let member_name = *(*parent).offsettomember.add(idx as usize);
        let existing = crate::functions::lua_h_getstr::lua_h_getstr(
            (*child).memberstooffset,
            member_name,
        );
        if !ttisnil!(existing) {
            lua_g_runerror!(
                L,
                "Cannot override instance member '{}' of parent class '{}' in child class '{}'",
                string_value(member_name),
                string_value((*parent).name),
                string_value((*child).name)
            );
        }
    }

    let new_class =
        crate::functions::lua_r_newblankclass::lua_r_newblankclass(L, (*child).name);
    let mut num_static_members_to_copy = 0u32;
    for idx in (*parent).numberofinstancemembers..(*parent).numberofallmembers {
        let member_name = *(*parent).offsettomember.add(idx as usize);
        let existing = crate::functions::lua_h_getstr::lua_h_getstr(
            (*child).memberstooffset,
            member_name,
        );
        if ttisnil!(existing) {
            num_static_members_to_copy += 1;
        }
    }

    let num_members = (*child).numberofallmembers
        + (*parent).numberofinstancemembers
        + num_static_members_to_copy;
    (*new_class).offsettomember =
        luaM_newarray!(L, num_members, *mut TString, (*new_class).memcat);
    (*new_class).numberofallmembers = num_members;
    (*new_class).memberstooffset =
        crate::functions::lua_h_new::lua_h_new(L, 0, num_members as i32);
    luaC_objbarrier!(L, new_class, (*new_class).memberstooffset);

    let mut offset = 0u32;
    while offset < (*parent).numberofinstancemembers {
        let member_name = *(*parent).offsettomember.add(offset as usize);
        *(*new_class).offsettomember.add(offset as usize) = member_name;
        let value = crate::functions::lua_h_setstr::lua_h_setstr(
            L,
            (*new_class).memberstooffset,
            member_name,
        );
        setnvalue!(value, offset as f64);
        luaC_barrier!(L, (*new_class).memberstooffset, value as *const TValue);
        offset += 1;
    }

    for idx in 0..(*child).numberofinstancemembers {
        let member_name = *(*child).offsettomember.add(idx as usize);
        *(*new_class).offsettomember.add(offset as usize) = member_name;
        let value = crate::functions::lua_h_setstr::lua_h_setstr(
            L,
            (*new_class).memberstooffset,
            member_name,
        );
        setnvalue!(value, offset as f64);
        luaC_barrier!(L, (*new_class).memberstooffset, value as *const TValue);
        offset += 1;
    }

    (*new_class).staticmembers =
        luaM_newarray!(L, num_members - offset, TValue, (*new_class).memcat);
    (*new_class).numberofinstancemembers = offset;

    let mut num_static_members_copied = 0u32;
    for idx in (*parent).numberofinstancemembers..(*parent).numberofallmembers {
        let member_name = *(*parent).offsettomember.add(idx as usize);
        let existing = crate::functions::lua_h_getstr::lua_h_getstr(
            (*child).memberstooffset,
            member_name,
        );
        if ttisnil!(existing) {
            let parent_value = (*parent)
                .staticmembers
                .add((idx - (*parent).numberofinstancemembers) as usize);
            crate::functions::lua_r_registerstaticmember::lua_r_registerstaticmember(
                L,
                new_class,
                member_name,
                parent_value,
                offset,
                num_static_members_copied,
            );
            offset += 1;
            num_static_members_copied += 1;
        }
    }

    for idx in (*child).numberofinstancemembers..(*child).numberofallmembers {
        let member_name = *(*child).offsettomember.add(idx as usize);
        let child_value = (*child)
            .staticmembers
            .add((idx - (*child).numberofinstancemembers) as usize);
        crate::functions::lua_r_registerstaticmember::lua_r_registerstaticmember(
            L,
            new_class,
            member_name,
            child_value,
            offset,
            num_static_members_copied,
        );
        offset += 1;
        num_static_members_copied += 1;
    }

    LUAU_ASSERT!(
        num_static_members_copied
            == num_static_members_to_copy
                + ((*child).numberofallmembers - (*child).numberofinstancemembers)
    );
    crate::functions::lua_r_addclassmetatable::lua_r_addclassmetatable(L, new_class);
    if !(*parent).instancemetatable.is_null() {
        (*new_class).instancemetatable = crate::functions::lua_h_clone::lua_h_clone(
            L,
            (*parent).instancemetatable,
        );
        luaC_objbarrier!(L, new_class, (*new_class).instancemetatable);
    } else {
        (*new_class).instancemetatable = core::ptr::null_mut();
    }

    new_class
}
