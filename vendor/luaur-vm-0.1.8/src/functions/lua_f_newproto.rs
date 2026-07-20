use crate::enums::lua_type::lua_Type;
use crate::macros::lua_c_init::luaC_init;
use crate::records::proto::Proto;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn luaF_newproto(l: *mut lua_State) -> *mut Proto {
    let f = crate::functions::lua_m_newgco::luaM_newgco_(
        l,
        core::mem::size_of::<Proto>(),
        (*l).activememcat,
    ) as *mut Proto;

    luaC_init!(l, f, lua_Type::LUA_TPROTO as c_int);

    (*f).nups = 0;
    (*f).numparams = 0;
    (*f).is_vararg = 0;
    (*f).maxstacksize = 0;
    (*f).flags = 0;

    (*f).k = core::ptr::null_mut();
    (*f).code = core::ptr::null_mut();
    (*f).p = core::ptr::null_mut();
    (*f).codeentry = core::ptr::null();

    (*f).execdata = core::ptr::null_mut();
    (*f).exectarget = 0;

    (*f).lineinfo = core::ptr::null_mut();
    (*f).abslineinfo = core::ptr::null_mut();
    (*f).locvars = core::ptr::null_mut();
    (*f).upvalues = core::ptr::null_mut();
    (*f).source = core::ptr::null_mut();

    (*f).debugname = core::ptr::null_mut();
    (*f).debuginsn = core::ptr::null_mut();

    (*f).typeinfo = core::ptr::null_mut();
    (*f).userdata = core::ptr::null_mut();
    (*f).gclist = core::ptr::null_mut();

    (*f).sizecode = 0;
    (*f).sizep = 0;
    (*f).sizelocvars = 0;
    (*f).sizeupvalues = 0;
    (*f).sizek = 0;
    (*f).sizelineinfo = 0;
    (*f).linegaplog2 = 0;
    (*f).linedefined = 0;
    (*f).bytecodeid = 0;
    (*f).sizetypeinfo = 0;

    (*f).feedbackvec = core::ptr::null_mut();
    (*f).feedbackvecsize = 0;
    (*f).funid = 0;

    f
}

#[allow(unused_imports)]
pub use luaF_newproto as lua_f_newproto;
