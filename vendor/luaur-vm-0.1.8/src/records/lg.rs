#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(C)]
pub struct Lg {
    pub l: crate::records::lua_state::lua_State,
    pub g: crate::records::global_state::global_State,
}

#[allow(non_camel_case_types)]
pub type LG = Lg;
