use crate::functions::lua_d_pcall::luaD_pcall;
use crate::functions::lua_gettop::lua_gettop;
use crate::macros::savestack::savestack;
use crate::macros::setnilvalue::setnilvalue;
use crate::records::resolve_import::ResolveImport;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn resolve_import_safe(
    L: *mut lua_State,
    env: *mut LuaTable,
    k: *mut TValue,
    id: u32,
) {
    let mut ri = ResolveImport { k, id };

    if (*env).safeenv != 0 {
        let old_top = lua_gettop(L);
        let status = luaD_pcall(
            L,
            Some(ResolveImport::run),
            &mut ri as *mut _ as *mut core::ffi::c_void,
            savestack!(L, (*L).top) as isize,
            0,
        );

        LUAU_ASSERT!(old_top + 1 == lua_gettop(L));

        if status != 0 {
            setnilvalue!((*L).top.sub(1));
        }
    } else {
        setnilvalue!((*L).top);
        (*L).top = (*L).top.add(1);
    }
}
