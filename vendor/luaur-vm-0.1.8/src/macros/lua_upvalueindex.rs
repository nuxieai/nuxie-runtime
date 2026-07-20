pub const LUAI_MAXCSTACK: i32 = 8000;

pub const LUA_GLOBALSINDEX: i32 = -LUAI_MAXCSTACK - 2002;

#[inline(always)]
pub const fn lua_upvalueindex(i: i32) -> i32 {
    LUA_GLOBALSINDEX - i
}
