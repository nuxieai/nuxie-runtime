//! `LUAU_FLAGVERSION(flag, version)` — stamps a version onto a previously-defined
//! flag. Reference: `luau/Common/include/Luau/Common.h`
//! (`static_assert(version != 0)` + a `static FValueVersionSetter` that runs at
//! init).
//!
//! The compile-time `version != 0` check is preserved. The runtime stamping is a
//! side-effecting walk ([`crate::records::f_value_version_setter::FValueVersionSetter::new`]),
//! which cannot run in a Rust `static` initializer (no static-ctor side effects).
//! It is deferred to the same startup registration pass that calls
//! [`crate::records::f_value::FValue::register`] — see the deviation note in
//! [`crate::records::f_value`]. (Not yet needed: nothing reads a flag's version.)

#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_FLAGVERSION {
    ($flag:ident, $version:expr) => {
        const _: () = {
            assert!($version != 0, "LUAU_FLAGVERSION version cannot be 0");
        };
    };
}

pub use LUAU_FLAGVERSION;
