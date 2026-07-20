use crate::enums::lua_type::lua_Type;
use crate::functions::validateclass::validateclass;
use crate::functions::validateclosure::validateclosure;
use crate::functions::validateobject::validateobject;
use crate::functions::validateobjref::validateobjref;
use crate::functions::validateproto::validateproto;
use crate::functions::validateref::validateref;
use crate::functions::validatestack::validatestack;
use crate::functions::validatetable::validatetable;
use crate::macros::gco_2_cl::gco2cl;
use crate::macros::gco_2_class::gco2class;
use crate::macros::gco_2_h::gco2h;
use crate::macros::gco_2_object::gco2object;
use crate::macros::gco_2_p::gco2p;
use crate::macros::gco_2_th::gco2th;
use crate::macros::gco_2_u::gco2u;
use crate::macros::gco_2_uv::gco2uv;
use crate::macros::isdead::isdead;
use crate::macros::obj_2_gco::obj2gco;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn validateobj(g: *mut global_State, o: *mut GCObject) {
    if isdead!(g, o) {
        LUAU_ASSERT!((*g).gcstate == 4);
        return;
    }

    match (*o).gch.tt as i32 {
        t if t == lua_Type::LUA_TSTRING as i32 => {}
        t if t == lua_Type::LUA_TTABLE as i32 => {
            validatetable(g, gco2h!(o) as *mut _);
        }
        t if t == lua_Type::LUA_TFUNCTION as i32 => {
            validateclosure(g, gco2cl!(o) as *mut _);
        }
        t if t == lua_Type::LUA_TUSERDATA as i32 => {
            let u = gco2u!(o) as *const _ as *mut crate::records::udata::Udata;
            if !(*u).metatable.is_null() {
                validateobjref(g, o, obj2gco!((*u).metatable));
            }
        }
        t if t == lua_Type::LUA_TTHREAD as i32 => {
            validatestack(g, gco2th!(o) as *mut _);
        }
        t if t == lua_Type::LUA_TBUFFER as i32 => {}
        t if t == lua_Type::LUA_TPROTO as i32 => {
            validateproto(g, gco2p!(o) as *mut _);
        }
        t if t == lua_Type::LUA_TUPVAL as i32 => {
            let uv = gco2uv!(o) as *const _ as *mut crate::records::up_val::UpVal;
            validateref(g, o, (*uv).v);
        }
        t if t == lua_Type::LUA_TCLASS as i32 => {
            validateclass(g, gco2class!(o) as *mut _);
        }
        t if t == lua_Type::LUA_TOBJECT as i32 => {
            validateobject(g, gco2object!(o) as *mut _);
        }
        _ => {
            LUAU_ASSERT!(false);
        }
    }
}
