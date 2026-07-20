use crate::functions::lua_createtable::lua_createtable;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_setfield::lua_setfield;
use crate::functions::lua_setmetatable::lua_setmetatable;
use crate::macros::lua_pop::lua_pop;
use crate::macros::lua_pushliteral::LUA_PUSHLITERAL;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn createmetatable_mut(l: *mut lua_State) {
    lua_createtable(l, 0, 1); // create metatable for strings

    LUA_PUSHLITERAL(l as *mut core::ffi::c_void, c"".as_ptr()); // dummy string

    lua_pushvalue(l, -2);

    lua_setmetatable(l, -2); // set string metatable

    lua_pop(l, 1); // pop dummy string

    lua_pushvalue(l, -2); // string library...

    lua_setfield(l, -2, c"__index".as_ptr()); // ...is the __index metamethod

    lua_pop(l, 1); // pop metatable
}
