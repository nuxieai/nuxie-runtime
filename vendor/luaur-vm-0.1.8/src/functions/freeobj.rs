use crate::enums::lua_type::lua_Type;
use crate::functions::lua_b_freebuffer::lua_b_freebuffer;
use crate::functions::lua_e_freethread::luaE_freethread;
use crate::functions::lua_f_freeclosure::lua_f_freeclosure;
use crate::functions::lua_f_freeproto::luaF_freeproto;
use crate::functions::lua_f_freeupval::lua_f_freeupval;
use crate::functions::lua_h_free::lua_h_free;
use crate::functions::lua_r_freeclass::lua_r_freeclass;
use crate::functions::lua_r_freeobject::lua_r_freeobject;
use crate::functions::lua_s_free::luaS_free;
use crate::functions::lua_u_freeudata::lua_u_freeudata;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn freeobj(l: *mut lua_State, o: *mut GCObject, page: *mut lua_Page) {
    match (*o).gch.tt as i32 {
        x if x == lua_Type::LUA_TPROTO as i32 => {
            luaF_freeproto(l, core::ptr::addr_of_mut!((*o).p) as *mut _, page);
        }
        x if x == lua_Type::LUA_TFUNCTION as i32 => {
            lua_f_freeclosure(l, core::ptr::addr_of_mut!((*o).cl) as *mut _, page);
        }
        x if x == lua_Type::LUA_TUPVAL as i32 => {
            lua_f_freeupval(l, core::ptr::addr_of_mut!((*o).uv) as *mut _, page);
        }
        x if x == lua_Type::LUA_TTABLE as i32 => {
            lua_h_free(l, core::ptr::addr_of_mut!((*o).h) as *mut _, page);
        }
        x if x == lua_Type::LUA_TTHREAD as i32 => {
            let th = core::ptr::addr_of_mut!((*o).th) as *mut lua_State;
            LUAU_ASSERT!(th != l && th != (*(*l).global).mainthread);
            luaE_freethread(l, th, page);
        }
        x if x == lua_Type::LUA_TSTRING as i32 => {
            luaS_free(l, core::ptr::addr_of_mut!((*o).ts) as *mut _, page);
        }
        x if x == lua_Type::LUA_TUSERDATA as i32 => {
            lua_u_freeudata(l, core::ptr::addr_of_mut!((*o).u) as *mut _, page);
        }
        x if x == lua_Type::LUA_TBUFFER as i32 => {
            lua_b_freebuffer(l, core::ptr::addr_of_mut!((*o).buf) as *mut _, page);
        }
        x if x == lua_Type::LUA_TCLASS as i32 => {
            lua_r_freeclass(l, core::ptr::addr_of_mut!((*o).lclass) as *mut _, page);
        }
        x if x == lua_Type::LUA_TOBJECT as i32 => {
            lua_r_freeobject(l, core::ptr::addr_of_mut!((*o).lobject) as *mut _, page);
        }
        _ => {
            LUAU_ASSERT!(false);
        }
    }
}
