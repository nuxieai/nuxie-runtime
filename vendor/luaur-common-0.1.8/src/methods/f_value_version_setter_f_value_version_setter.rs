//! `FValueVersionSetter::FValueVersionSetter(const char* name, unsigned version)`.
//! Reference: `luau/Common/include/Luau/Common.h`. Walks the `bool` and `int`
//! flag lists, stamping `version` onto every flag whose name matches, and asserts
//! at least one did (the `LUAU_FLAGVERSION` "must appear after the flag
//! definition" guard).

use core::ffi::{c_char, c_uint};
use core::sync::atomic::Ordering;

use crate::records::f_value::{FValue, FValueList};
use crate::records::f_value_version_setter::FValueVersionSetter;

impl FValueVersionSetter {
    /// # Safety
    /// Walks the global flag lists via raw pointers; call after the named flag's
    /// `register()` (the C++ macro guarantees this ordering within a source file).
    pub unsafe fn new(name: *const c_char, version: c_uint) -> Self {
        debug_assert!(version != 0, "LUAU_FLAGVERSION version cannot be 0");
        let mut found = false;

        let mut p = <bool as FValueList>::head().load(Ordering::Relaxed) as *const FValue<bool>;
        while !p.is_null() {
            if cstr_eq((*p).name, name) {
                (*p).set_version(version);
                found = true;
            }
            p = *(*p).next.get();
        }

        let mut q = <i32 as FValueList>::head().load(Ordering::Relaxed) as *const FValue<i32>;
        while !q.is_null() {
            if cstr_eq((*q).name, name) {
                (*q).set_version(version);
                found = true;
            }
            q = *(*q).next.get();
        }

        debug_assert!(
            found,
            "LUAU_FLAGVERSION must appear after the flag definition in the same source file"
        );

        FValueVersionSetter
    }
}

/// `strcmp(a, b) == 0` on nul-terminated C strings.
unsafe fn cstr_eq(a: *const c_char, b: *const c_char) -> bool {
    let mut i = 0isize;
    loop {
        let ca = *a.offset(i);
        let cb = *b.offset(i);
        if ca != cb {
            return false;
        }
        if ca == 0 {
            return true;
        }
        i += 1;
    }
}
