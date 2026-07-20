use crate::functions::lua_s_newlstr::luaS_newlstr;
use crate::functions::luai_num_2_str::luai_num2str;
use crate::macros::luai_maxnum_2_str::LUAI_MAXNUM2STR;
use crate::macros::nvalue::nvalue;
use crate::macros::setsvalue::setsvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub fn lua_v_tostring(L: *mut LuaState, obj: StkId) -> i32 {
    unsafe {
        if !ttisnumber!(obj) {
            0
        } else {
            let mut s = [0 as core::ffi::c_char; LUAI_MAXNUM2STR as usize];
            let n = nvalue!(obj);
            let e = luai_num2str(s.as_mut_ptr(), n);
            LUAU_ASSERT!((e as usize) < (s.as_ptr() as usize + s.len()));
            setsvalue!(
                L,
                obj,
                luaS_newlstr(L, s.as_ptr(), e.offset_from(s.as_ptr()) as usize)
            );
            1
        }
    }
}
