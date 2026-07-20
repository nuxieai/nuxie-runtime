use crate::enums::lua_type::lua_Type;
use crate::functions::dumpbuffer::dumpbuffer;
use crate::functions::dumpclass::dumpclass;
use crate::functions::dumpclosure::dumpclosure;
use crate::functions::dumpobject::dumpobject;
use crate::functions::dumpproto::dumpproto;
use crate::functions::dumpstring::dumpstring;
use crate::functions::dumptable::dumptable;
use crate::functions::dumpthread::dumpthread;
use crate::functions::dumpudata::dumpudata;
use crate::functions::dumpupval::dumpupval;
use crate::macros::gco_2_buf::gco2buf;
use crate::macros::gco_2_cl::gco2cl;
use crate::macros::gco_2_class::gco2class;
use crate::macros::gco_2_h::gco2h;
use crate::macros::gco_2_object::gco2object;
use crate::macros::gco_2_p::gco2p;
use crate::macros::gco_2_th::gco2th;
use crate::macros::gco_2_ts::gco2ts;
use crate::macros::gco_2_u::gco2u;
use crate::macros::gco_2_uv::gco2uv;
use crate::records::gc_object::GCObject;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn dumpobj(f: *mut core::ffi::c_void, o: *mut GCObject) {
    match (*o).gch.tt as i32 {
        t if t == lua_Type::LUA_TSTRING as i32 => {
            dumpstring(f, gco2ts!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TTABLE as i32 => {
            dumptable(f, gco2h!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TFUNCTION as i32 => {
            dumpclosure(f, gco2cl!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TUSERDATA as i32 => {
            dumpudata(f, gco2u!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TTHREAD as i32 => {
            dumpthread(f, gco2th!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TBUFFER as i32 => {
            dumpbuffer(f, gco2buf!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TCLASS as i32 => {
            dumpclass(f, gco2class!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TOBJECT as i32 => {
            dumpobject(f, gco2object!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TPROTO as i32 => {
            dumpproto(f, gco2p!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TUPVAL as i32 => {
            dumpupval(f, gco2uv!(o) as *const _ as *mut _);
        }
        _ => {
            LUAU_ASSERT!(false);
        }
    }
}
