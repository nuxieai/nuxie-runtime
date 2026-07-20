//! `FValue<T>::FValue(const char* name, T def, bool dynamic)`. Reference:
//! `luau/Common/include/Luau/Common.h`.
//!
//! Field init only — a `const fn` so flags can be `static`. The C++ ctor's
//! `list = this` side effect is [`FValue::register`] (see the deviation note in
//! [`crate::records::f_value`]).

use core::cell::UnsafeCell;
use core::ffi::c_char;

use crate::records::f_value::FValue;

impl<T: Copy> FValue<T> {
    pub const fn new(name: *const c_char, def: T, dynamic: bool) -> Self {
        FValue {
            value: UnsafeCell::new(def),
            dynamic,
            name,
            next: UnsafeCell::new(core::ptr::null()),
            version: UnsafeCell::new(0),
        }
    }
}
