#[allow(non_snake_case)]
#[macro_export]
macro_rules! ttype {
    ($o:expr) => {
        $crate::macros::ttype::TTypeOf::ttype($o)
    };
}

pub use ttype;

pub trait TTypeOf {
    fn ttype(self) -> core::ffi::c_int;
}

impl TTypeOf for *const crate::records::lua_t_value::TValue {
    fn ttype(self) -> core::ffi::c_int {
        unsafe { (*self).tt() }
    }
}

impl TTypeOf for *mut crate::records::lua_t_value::TValue {
    fn ttype(self) -> core::ffi::c_int {
        unsafe { (*self).tt() }
    }
}

impl TTypeOf for &crate::records::lua_t_value::TValue {
    fn ttype(self) -> core::ffi::c_int {
        self.tt()
    }
}

impl TTypeOf for &mut crate::records::lua_t_value::TValue {
    fn ttype(self) -> core::ffi::c_int {
        self.tt()
    }
}

impl TTypeOf for *const crate::records::t_key::TKey {
    fn ttype(self) -> core::ffi::c_int {
        unsafe { (*self).tt() }
    }
}

impl TTypeOf for *mut crate::records::t_key::TKey {
    fn ttype(self) -> core::ffi::c_int {
        unsafe { (*self).tt() }
    }
}

impl TTypeOf for &crate::records::t_key::TKey {
    fn ttype(self) -> core::ffi::c_int {
        self.tt()
    }
}

impl TTypeOf for &mut crate::records::t_key::TKey {
    fn ttype(self) -> core::ffi::c_int {
        self.tt()
    }
}

macro_rules! impl_gc_ttype_direct {
    ($t:ty) => {
        impl TTypeOf for *const $t {
            fn ttype(self) -> core::ffi::c_int {
                unsafe { (*self).tt as core::ffi::c_int }
            }
        }

        impl TTypeOf for *mut $t {
            fn ttype(self) -> core::ffi::c_int {
                unsafe { (*self).tt as core::ffi::c_int }
            }
        }
    };
}

macro_rules! impl_gc_ttype_hdr {
    ($t:ty) => {
        impl TTypeOf for *const $t {
            fn ttype(self) -> core::ffi::c_int {
                unsafe { (*self).hdr.tt as core::ffi::c_int }
            }
        }

        impl TTypeOf for *mut $t {
            fn ttype(self) -> core::ffi::c_int {
                unsafe { (*self).hdr.tt as core::ffi::c_int }
            }
        }
    };
}

impl TTypeOf for *const crate::records::gc_object::GCObject {
    fn ttype(self) -> core::ffi::c_int {
        unsafe { (*self).gch.tt as core::ffi::c_int }
    }
}

impl TTypeOf for *mut crate::records::gc_object::GCObject {
    fn ttype(self) -> core::ffi::c_int {
        unsafe { (*self).gch.tt as core::ffi::c_int }
    }
}

impl_gc_ttype_hdr!(crate::records::t_string::TString);
impl_gc_ttype_hdr!(crate::records::closure::Closure);
impl_gc_ttype_hdr!(crate::records::lua_state::lua_State);
impl_gc_ttype_hdr!(crate::records::proto::Proto);
impl_gc_ttype_hdr!(crate::records::up_val::UpVal);

impl_gc_ttype_direct!(crate::records::lua_table::LuaTable);
impl_gc_ttype_direct!(crate::records::udata::Udata);
impl_gc_ttype_direct!(crate::records::luau_buffer::LuauBuffer);
impl_gc_ttype_direct!(crate::records::luau_class::LuauClass);
impl_gc_ttype_direct!(crate::records::luau_object::LuauObject);
