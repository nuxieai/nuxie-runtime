use crate::functions::lua_close::lua_close;
use crate::functions::lua_l_newstate::lua_l_newstate;
use crate::functions::lua_pushcclosurek::lua_pushcclosurek;
use crate::macros::clvalue::clvalue;
use crate::macros::getstr::getstr;
use crate::records::closure::CClosure;
use crate::type_aliases::lua_state::lua_State;

unsafe fn noop(_: *mut lua_State) -> core::ffi::c_int {
    0
}

#[test]
fn managed_debug_names_preserve_both_representations() {
    let old = luaur_common::FFlag::LuauManagedDebugNames.get();

    let check = |managed: bool| unsafe {
        luaur_common::FFlag::LuauManagedDebugNames.set(managed);
        let state = lua_l_newstate();
        assert!(!state.is_null());
        lua_pushcclosurek(state, Some(noop), c"rung8".as_ptr(), 0, None);
        let closure = clvalue!((*state).top.sub(1));
        let c = core::ptr::addr_of!((*closure).inner.c).cast::<CClosure>();
        if managed {
            assert!(!(*c).debugname.is_null());
            assert_eq!(core::ffi::CStr::from_ptr(getstr((*c).debugname)).to_bytes(), b"rung8");
            assert!((*c).debugname_DEPRECATED.is_null());
        } else {
            assert!((*c).debugname.is_null());
            assert_eq!(
                core::ffi::CStr::from_ptr((*c).debugname_DEPRECATED).to_bytes(),
                b"rung8"
            );
        }
        lua_close(state);
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        check(false);
        check(true);
    }));
    luaur_common::FFlag::LuauManagedDebugNames.set(old);
    result.expect("both debug-name representations should work");
}
