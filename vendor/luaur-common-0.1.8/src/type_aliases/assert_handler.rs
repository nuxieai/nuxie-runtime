use core::ffi::c_char;

#[allow(non_camel_case_types)]
pub type AssertHandler = Option<
    unsafe extern "C" fn(
        expression: *const c_char,
        file: *const c_char,
        line: i32,
        function: *const c_char,
    ) -> i32,
>;
