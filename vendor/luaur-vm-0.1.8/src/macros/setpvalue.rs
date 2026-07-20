//! Source: `VM/src/lobject.h:142` (hand-ported)
// #define setpvalue(obj, x, tag)
//     { TValue* i_o = (obj); i_o->value.p = (x); i_o->extra[0] = (tag); i_o->tt = LUA_TLIGHTUSERDATA; }
#[allow(non_snake_case)]
#[macro_export]
macro_rules! setpvalue {
    ($obj:expr, $x:expr, $tag:expr) => {{
        let i_o = $obj;
        (*i_o).value.p = $x;
        (*i_o).extra[0] = $tag;
        (*i_o).set_tt($crate::enums::lua_type::lua_Type::LUA_TLIGHTUSERDATA as i32);
    }};
}
pub use setpvalue;
