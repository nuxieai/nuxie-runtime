#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(C)]
pub struct TempBuffer<T> {
    pub(crate) L: *mut crate::type_aliases::lua_state::lua_State,
    pub(crate) data: *mut T,
    pub(crate) count: usize,
}
