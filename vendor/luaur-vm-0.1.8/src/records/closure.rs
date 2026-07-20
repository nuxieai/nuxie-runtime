#[allow(non_camel_case_types)]
#[repr(C)]
pub struct CClosure {
    pub f: crate::type_aliases::lua_c_function::lua_CFunction,
    pub cont: crate::type_aliases::lua_continuation::lua_Continuation,
    pub debugname: *const core::ffi::c_char,
    pub upvals: [crate::type_aliases::t_value::TValue; 1],
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct LClosure {
    pub p: *mut crate::records::proto::Proto,
    pub uprefs: [crate::type_aliases::t_value::TValue; 1],
}

#[allow(non_snake_case)]
#[repr(C)]
pub union ClosureInner {
    pub c: core::mem::ManuallyDrop<CClosure>,
    pub l: core::mem::ManuallyDrop<LClosure>,
}

impl core::fmt::Debug for ClosureInner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ClosureInner").finish_non_exhaustive()
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct Closure {
    pub hdr: crate::records::g_cheader::GCheader,
    pub isC: u8,
    pub nupvalues: u8,
    pub stacksize: u8,
    pub preload: u8,
    pub usage: u64,
    pub gclist: *mut crate::records::gc_object::GcObject,
    pub env: *mut crate::records::lua_table::LuaTable,
    pub inner: ClosureInner,
}

#[allow(non_camel_case_types)]
pub type closure = Closure;
