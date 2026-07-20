use crate::enums::lua_type::lua_Type;
use crate::functions::enumbuffer::enumbuffer;
use crate::functions::enumclass::enumclass;
use crate::functions::enumclosure::enumclosure;
use crate::functions::enumobject::enumobject;
use crate::functions::enumproto::enumproto;
use crate::functions::enumstring::enumstring;
use crate::functions::enumtable::enumtable;
use crate::functions::enumthread::enumthread;
use crate::functions::enumudata::enumudata;
use crate::functions::enumupval::enumupval;
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
use crate::records::enum_context::EnumContext;
use crate::records::gc_object::GCObject;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn enumobj(ctx: *mut EnumContext, o: *mut GCObject) {
    match (*o).gch.tt as i32 {
        t if t == lua_Type::LUA_TSTRING as i32 => {
            enumstring(ctx, gco2ts!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TTABLE as i32 => {
            enumtable(ctx, gco2h!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TFUNCTION as i32 => {
            enumclosure(ctx, gco2cl!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TUSERDATA as i32 => {
            enumudata(ctx, gco2u!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TTHREAD as i32 => {
            enumthread(ctx, gco2th!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TBUFFER as i32 => {
            enumbuffer(ctx, gco2buf!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TCLASS as i32 => {
            enumclass(ctx, gco2class!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TOBJECT as i32 => {
            enumobject(ctx, gco2object!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TPROTO as i32 => {
            enumproto(ctx, gco2p!(o) as *const _ as *mut _);
        }
        t if t == lua_Type::LUA_TUPVAL as i32 => {
            enumupval(ctx, gco2uv!(o) as *const _ as *mut _);
        }
        _ => {
            LUAU_ASSERT!(false);
        }
    }
}
