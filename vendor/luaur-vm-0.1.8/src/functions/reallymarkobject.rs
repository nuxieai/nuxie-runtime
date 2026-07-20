//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:239:reallymarkobject`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:239-309, hand-ported)

use crate::enums::lua_type::lua_Type;
use crate::macros::gco_2_cl::gco2cl;
use crate::macros::gco_2_class::gco2class;
use crate::macros::gco_2_h::gco2h;
use crate::macros::gco_2_object::gco2object;
use crate::macros::gco_2_p::gco2p;
use crate::macros::gco_2_th::gco2th;
use crate::macros::gco_2_u::gco2u;
use crate::macros::gco_2_uv::gco2uv;
use crate::macros::gray_2_black::gray2black;
use crate::macros::isdead::isdead;
use crate::macros::iswhite::iswhite;
use crate::macros::markobject::markobject;
use crate::macros::markvalue::markvalue;
use crate::macros::upisopen::upisopen;
use crate::macros::white_2_gray::white2gray;
use crate::records::closure::Closure;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use crate::records::lua_table::LuaTable;
use crate::records::luau_class::LuauClass;
use crate::records::luau_object::LuauObject;
use crate::records::proto::Proto;
use crate::records::up_val::UpVal;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn reallymarkobject(g: *mut global_State, o: *mut GCObject) {
    LUAU_ASSERT!(iswhite!(o) && !isdead!(g, o));
    white2gray!(o);
    match (*o).gch.tt as i32 {
        t if t == lua_Type::LUA_TSTRING as i32 => {}
        t if t == lua_Type::LUA_TUSERDATA as i32 => {
            let mt: *mut LuaTable = (*gco2u!(o)).metatable;
            gray2black!(o); // udata are never gray
            if !mt.is_null() {
                markobject!(g, mt);
            }
        }
        t if t == lua_Type::LUA_TUPVAL as i32 => {
            let uv = gco2uv!(o) as *const _ as *mut UpVal;
            markvalue!(g, (*uv).v);
            if !upisopen!(uv) {
                // closed?
                gray2black!(o); // open upvalues are never black
            }
        }
        t if t == lua_Type::LUA_TFUNCTION as i32 => {
            (*(gco2cl!(o) as *const _ as *mut Closure)).gclist = (*g).gray;
            (*g).gray = o;
        }
        t if t == lua_Type::LUA_TTABLE as i32 => {
            (*(gco2h!(o) as *const _ as *mut LuaTable)).gclist = (*g).gray;
            (*g).gray = o;
        }
        t if t == lua_Type::LUA_TTHREAD as i32 => {
            (*(gco2th!(o) as *const _ as *mut lua_State)).gclist = (*g).gray;
            (*g).gray = o;
        }
        t if t == lua_Type::LUA_TBUFFER as i32 => {
            gray2black!(o); // buffers are never gray
        }
        t if t == lua_Type::LUA_TPROTO as i32 => {
            (*(gco2p!(o) as *const _ as *mut Proto)).gclist = (*g).gray;
            (*g).gray = o;
        }
        t if t == lua_Type::LUA_TCLASS as i32 => {
            (*(gco2class!(o) as *const _ as *mut LuauClass)).gclist = (*g).gray;
            (*g).gray = o;
        }
        t if t == lua_Type::LUA_TOBJECT as i32 => {
            (*(gco2object!(o) as *const _ as *mut LuauObject)).gclist = (*g).gray;
            (*g).gray = o;
        }
        _ => {
            LUAU_ASSERT!(false);
        }
    }
}
