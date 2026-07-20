#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_ASSERT {
    ($expr:expr) => {
        if $crate::macros::luau_assertenabled::LUAU_ASSERTENABLED {
            if !($expr) {
                // `assertCallHandler` takes C strings; `stringify!`/`file!`
                // produce string literals, so nul-terminate them to hand over a
                // valid `*const c_char`. (`__FUNCTION__` has no stable Rust
                // equivalent, so the function name is a placeholder.)
                $crate::functions::assert_call_handler::assert_call_handler(
                    concat!(stringify!($expr), "\0").as_ptr() as *const core::ffi::c_char,
                    concat!(file!(), "\0").as_ptr() as *const core::ffi::c_char,
                    line!() as i32,
                    c"unknown".as_ptr(),
                );
                $crate::LUAU_DEBUGBREAK!();
            }
        }
    };
    // Tolerant 2-arg form: the C++ `LUAU_ASSERT(cond && "message")` idiom (and
    // `LUAU_ASSERT(!"message")`) routinely lands from the model as a two-token
    // call `LUAU_ASSERT!(cond, "message")`. Accept it — assert the condition and
    // carry the message into the assertion text — rather than fail compilation
    // over a separator. Mirrors the trailing-comma / camelCase tolerances the
    // foundation already grants machine translations.
    ($expr:expr, $msg:expr) => {
        if $crate::macros::luau_assertenabled::LUAU_ASSERTENABLED {
            if !($expr) {
                $crate::functions::assert_call_handler::assert_call_handler(
                    concat!(stringify!($expr), " : ", stringify!($msg), "\0").as_ptr()
                        as *const core::ffi::c_char,
                    concat!(file!(), "\0").as_ptr() as *const core::ffi::c_char,
                    line!() as i32,
                    c"unknown".as_ptr(),
                );
                $crate::LUAU_DEBUGBREAK!();
            }
        }
    };
}

pub use LUAU_ASSERT;
