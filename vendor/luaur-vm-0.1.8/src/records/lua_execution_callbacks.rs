use crate::records::closure::Closure;
use crate::records::lua_state::lua_State;
use crate::records::proto::Proto;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct lua_ExecutionCallbacks {
    pub context: *mut core::ffi::c_void,
    pub close: Option<unsafe extern "C" fn(L: *mut lua_State)>,
    pub destroy: Option<unsafe extern "C" fn(L: *mut lua_State, proto: *mut Proto)>,
    pub enter:
        Option<unsafe extern "C" fn(L: *mut lua_State, proto: *mut Proto) -> core::ffi::c_int>,
    pub disable: Option<unsafe extern "C" fn(L: *mut lua_State, proto: *mut Proto)>,
    pub getmemorysize: Option<unsafe extern "C" fn(L: *mut lua_State, proto: *mut Proto) -> usize>,
    pub gettypemapping: Option<
        unsafe extern "C" fn(L: *mut lua_State, str: *const core::ffi::c_char, len: usize) -> u8,
    >,
    pub getcounterdata: Option<
        unsafe extern "C" fn(
            L: *mut lua_State,
            proto: *mut Proto,
            count: *mut usize,
        ) -> *mut core::ffi::c_char,
    >,
    pub inlinefunction: Option<
        unsafe extern "C" fn(
            L: *mut lua_State,
            caller: *mut Closure,
            target: *mut Closure,
            pc: u32,
        ) -> *mut Proto,
    >,
}

#[allow(non_camel_case_types)]
pub type LuaExecutionCallbacks = lua_ExecutionCallbacks;
