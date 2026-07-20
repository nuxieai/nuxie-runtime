use crate::functions::lua_m_free::luaM_free_;
use crate::functions::lua_m_freegco::luaM_freegco_;
use crate::records::feedback_vector_slot::FeedbackVectorSlot;
use crate::records::gc_object::GCObject;
use crate::records::loc_var::LocVar;
use crate::records::lua_page::lua_Page;
use crate::records::proto::Proto;
use crate::records::t_string::TString;
use crate::type_aliases::instruction::Instruction;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luaF_freeproto(l: *mut lua_State, f: *mut Proto, page: *mut lua_Page) {
    luaM_free_(
        l,
        (*f).code as *mut core::ffi::c_void,
        (*f).sizecode as usize * core::mem::size_of::<Instruction>(),
        (*f).hdr.memcat,
    );
    luaM_free_(
        l,
        (*f).p as *mut core::ffi::c_void,
        (*f).sizep as usize * core::mem::size_of::<*mut Proto>(),
        (*f).hdr.memcat,
    );
    luaM_free_(
        l,
        (*f).k as *mut core::ffi::c_void,
        (*f).sizek as usize * core::mem::size_of::<TValue>(),
        (*f).hdr.memcat,
    );
    if !(*f).lineinfo.is_null() {
        luaM_free_(
            l,
            (*f).lineinfo as *mut core::ffi::c_void,
            (*f).sizelineinfo as usize * core::mem::size_of::<u8>(),
            (*f).hdr.memcat,
        );
    }
    luaM_free_(
        l,
        (*f).locvars as *mut core::ffi::c_void,
        (*f).sizelocvars as usize * core::mem::size_of::<LocVar>(),
        (*f).hdr.memcat,
    );
    luaM_free_(
        l,
        (*f).upvalues as *mut core::ffi::c_void,
        (*f).sizeupvalues as usize * core::mem::size_of::<*mut TString>(),
        (*f).hdr.memcat,
    );
    if !(*f).debuginsn.is_null() {
        luaM_free_(
            l,
            (*f).debuginsn as *mut core::ffi::c_void,
            (*f).sizecode as usize * core::mem::size_of::<u8>(),
            (*f).hdr.memcat,
        );
    }

    if !(*f).execdata.is_null() {
        if let Some(destroy) = (*(*l).global).ecb.destroy {
            destroy(l, f);
        }
    }

    if !(*f).typeinfo.is_null() {
        luaM_free_(
            l,
            (*f).typeinfo as *mut core::ffi::c_void,
            (*f).sizetypeinfo as usize * core::mem::size_of::<u8>(),
            (*f).hdr.memcat,
        );
    }

    if !(*f).feedbackvec.is_null() {
        luaM_free_(
            l,
            (*f).feedbackvec as *mut core::ffi::c_void,
            (*f).feedbackvecsize as usize * core::mem::size_of::<FeedbackVectorSlot>(),
            (*f).hdr.memcat,
        );
    }

    luaM_freegco_(
        l,
        f as *mut GCObject,
        core::mem::size_of::<Proto>(),
        (*f).hdr.memcat,
        page,
    );
}

#[allow(unused_imports)]
pub use luaF_freeproto as lua_f_freeproto;
