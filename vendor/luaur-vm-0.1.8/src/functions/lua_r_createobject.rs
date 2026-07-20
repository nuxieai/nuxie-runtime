use crate::enums::lua_type::lua_Type;
use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_l_checktype::lua_l_checktype;
use crate::macros::getstr::getstr;
use crate::macros::lua_c_barrierfast::luaC_barrierfast;
use crate::macros::lua_c_init::luaC_init;
use crate::macros::lua_m_newarray::luaM_newarray;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::setobj::setobj;
use crate::macros::setobjectvalue::setobjectvalue;
use crate::macros::setsvalue::setsvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_r_createobject(L: *mut lua_State) -> core::ffi::c_int {
    lua_l_checktype(L, 1, lua_Type::LUA_TCLASS as core::ffi::c_int);
    let classobject = core::ptr::addr_of_mut!((*(*(*L).base).value.gc).lclass)
        as *mut crate::records::luau_class::LuauClass;
    let classinst = crate::functions::lua_m_newgco::luaM_newgco_(
        L,
        core::mem::size_of::<crate::records::luau_object::LuauObject>(),
        (*L).activememcat,
    ) as *mut crate::records::luau_object::LuauObject;
    luaC_init!(L, classinst, lua_Type::LUA_TOBJECT as core::ffi::c_int);
    (*classinst).lclass = classobject;
    (*classinst).numberofmembers = (*classobject).numberofinstancemembers;
    (*classinst).members =
        luaM_newarray!(L, (*classinst).numberofmembers, TValue, (*L).activememcat);
    let numargs: core::ffi::c_int = lua_gettop(L);

    // We need to initialize all of the instance members to `nil` to start.
    for idx in 0..(*classobject).numberofinstancemembers as core::ffi::c_int {
        setnilvalue!((*classinst).members.add(idx as usize));
    }

    // Push the class object onto the stack. We do this prior to setting the
    // fields as we may reallocate the stack as part of indexing into the
    // second argument (if present).
    setobjectvalue!(L, (*L).top, classinst);
    (*L).top = (*L).top.wrapping_add(1);

    // Stack location to hold the table lookup result
    setnilvalue!((*L).top);
    (*L).top = (*L).top.wrapping_add(1);

    match numargs {
        1 => {
            // If given no second argument, assume all class members are `nil`.
        }
        2 => {
            // If given a second argument, use it to initialize all class members.
            for idx in 0..(*classobject).numberofinstancemembers as core::ffi::c_int {
                let mut key: TValue = TValue::default();
                setsvalue!(
                    L,
                    &mut key,
                    *(*classobject).offsettomember.add(idx as usize)
                );
                crate::functions::lua_v_gettable::lua_v_gettable(
                    L,
                    (*L).base.add(1),
                    &mut key,
                    (*L).top.wrapping_sub(1),
                );
                setobj!(
                    L,
                    (*classinst).members.add(idx as usize),
                    (*L).top.wrapping_sub(1)
                );
            }
        }
        _ => {
            crate::functions::lua_l_error_l::lua_l_error_l(
                L,
                c"wrong number of arguments for constructing a '%s'".as_ptr(),
                core::format_args!(
                    "wrong number of arguments for constructing a '{}'",
                    unsafe {
                        core::ffi::CStr::from_ptr(getstr((*classobject).name)).to_string_lossy()
                    }
                ),
            );
        }
    }

    (*L).top = (*L).top.wrapping_sub(1);

    // Preserve the GC invariant, moving barrier back once after writing multiple objects (similar to SETLIST)
    luaC_barrierfast!(L, classinst);

    1
}
