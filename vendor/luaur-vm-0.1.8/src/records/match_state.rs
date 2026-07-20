#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MatchState {
    pub(crate) matchdepth: core::ffi::c_int,
    pub(crate) src_init: *const core::ffi::c_char,
    pub(crate) src_end: *const core::ffi::c_char,
    pub(crate) p_end: *const core::ffi::c_char,
    pub(crate) L: *mut crate::records::lua_state::LuaState,
    pub(crate) level: core::ffi::c_int,
    pub(crate) capture: [MatchState_capture; 32],
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MatchState_capture {
    pub(crate) init: *const core::ffi::c_char,
    pub(crate) len: isize,
}

impl Default for MatchState {
    fn default() -> Self {
        Self {
            matchdepth: 0,
            src_init: core::ptr::null(),
            src_end: core::ptr::null(),
            p_end: core::ptr::null(),
            L: core::ptr::null_mut(),
            level: 0,
            capture: [MatchState_capture {
                init: core::ptr::null(),
                len: 0,
            }; 32],
        }
    }
}

#[allow(non_camel_case_types)]
pub type match_state = MatchState;
