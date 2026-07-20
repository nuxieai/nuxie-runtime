use crate::enums::lua_type::lua_Type;
use crate::macros::gkey::{gkey, gval};
use crate::macros::iscollectable::iscollectable;
use crate::macros::setttype::setttype;
use crate::macros::ttisnil::ttisnil;
use crate::type_aliases::lua_node::LuaNode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub unsafe fn removeentry(n: *mut LuaNode) {
    LUAU_ASSERT!(ttisnil!(gval!(n)));
    if iscollectable!(gkey!(n)) {
        setttype!(gkey!(n), lua_Type::LUA_TDEADKEY as i32);
    }
}
