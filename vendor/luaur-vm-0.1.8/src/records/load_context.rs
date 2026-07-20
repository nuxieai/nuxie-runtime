use crate::records::proto::Proto;
use crate::records::t_string::TString;
use crate::records::temp_buffer::TempBuffer;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct LoadContext {
    pub(crate) strings: TempBuffer<*mut TString>,
    pub(crate) protos: TempBuffer<*mut Proto>,
    pub(crate) chunkname: *const core::ffi::c_char,
    pub(crate) data: *const core::ffi::c_char,
    pub(crate) size: usize,
    pub(crate) env: i32,
    pub(crate) result: i32,
}

impl LoadContext {
    #[allow(non_snake_case)]
    pub(crate) unsafe extern "C" fn run(L: lua_State, ud: *mut core::ffi::c_void) {
        let ctx = &mut *(ud as *mut LoadContext);

        ctx.result = loadsafe(
            L,
            &mut ctx.strings,
            &mut ctx.protos,
            ctx.chunkname,
            ctx.data,
            ctx.size,
            ctx.env,
        );
    }
}

extern "C" {
    fn loadsafe(
        L: lua_State,
        strings: *mut TempBuffer<*mut TString>,
        protos: *mut TempBuffer<*mut Proto>,
        chunkname: *const core::ffi::c_char,
        data: *const core::ffi::c_char,
        size: usize,
        env: i32,
    ) -> i32;
}
