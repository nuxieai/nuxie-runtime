use crate::functions::assert_handler::assert_handler;
use crate::macros::luau_noinline::LUAU_NOINLINE;
use core::ffi::c_char;

LUAU_NOINLINE! {
    pub fn assert_call_handler(
        expression: *const c_char,
        file: *const c_char,
        line: i32,
        function: *const c_char,
    ) -> i32 {
        let handler_ptr = assert_handler();
        if let Some(handler) = *handler_ptr {
            unsafe {
                return handler(expression, file, line, function);
            }
        }

        // No custom handler: print the assertion before LUAU_DEBUGBREAK traps the
        // process (matches the C++ default `assertCallHandler`, which writes the
        // message to stderr). Without this the failure is a silent `int 3`, which on
        // Windows surfaces only as a 0xC0000003 abort with no indication of which
        // assert fired — making platform-specific assertion failures undiagnosable.
        #[cfg(feature = "std")]
        unsafe {
            let expr = core::ffi::CStr::from_ptr(expression).to_string_lossy();
            let f = core::ffi::CStr::from_ptr(file).to_string_lossy();
            eprintln!("LUAU_ASSERT failed: {} ({}:{})", expr, f, line);
        }

        1
    }
}
