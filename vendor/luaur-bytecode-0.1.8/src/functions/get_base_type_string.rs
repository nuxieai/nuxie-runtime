use luaur_common::macros::luau_assert::LUAU_ASSERT;

// Values must match the LuauBytecodeType enum (luaur_common / C++ Bytecode.h):
// NIL=0, BOOLEAN=1, NUMBER=2, STRING=3, TABLE=4, FUNCTION=5, THREAD=6,
// USERDATA=7, VECTOR=8, BUFFER=9, INTEGER=10, ANY=15. The port had a spurious
// INTEGER=3 that shifted STRING..BUFFER down by one (USERDATA dumped as 'thread').
#[allow(non_upper_case_globals)]
const LBC_TYPE_NIL: u8 = 0;
#[allow(non_upper_case_globals)]
const LBC_TYPE_BOOLEAN: u8 = 1;
#[allow(non_upper_case_globals)]
const LBC_TYPE_NUMBER: u8 = 2;
#[allow(non_upper_case_globals)]
const LBC_TYPE_STRING: u8 = 3;
#[allow(non_upper_case_globals)]
const LBC_TYPE_TABLE: u8 = 4;
#[allow(non_upper_case_globals)]
const LBC_TYPE_FUNCTION: u8 = 5;
#[allow(non_upper_case_globals)]
const LBC_TYPE_THREAD: u8 = 6;
#[allow(non_upper_case_globals)]
const LBC_TYPE_USERDATA: u8 = 7;
#[allow(non_upper_case_globals)]
const LBC_TYPE_VECTOR: u8 = 8;
#[allow(non_upper_case_globals)]
const LBC_TYPE_BUFFER: u8 = 9;
#[allow(non_upper_case_globals)]
const LBC_TYPE_INTEGER: u8 = 10;
#[allow(non_upper_case_globals)]
const LBC_TYPE_ANY: u8 = 15;

#[allow(non_upper_case_globals)]
const LBC_TYPE_OPTIONAL_BIT: u8 = 128;

pub(crate) fn get_base_type_string(r#type: u8) -> *const core::ffi::c_char {
    let tag = r#type & !LBC_TYPE_OPTIONAL_BIT;

    match tag {
        LBC_TYPE_NIL => return c"nil".as_ptr(),
        LBC_TYPE_BOOLEAN => return c"boolean".as_ptr(),
        LBC_TYPE_NUMBER => return c"number".as_ptr(),
        LBC_TYPE_INTEGER => return c"integer".as_ptr(),
        LBC_TYPE_STRING => return c"string".as_ptr(),
        LBC_TYPE_TABLE => return c"table".as_ptr(),
        LBC_TYPE_FUNCTION => return c"function".as_ptr(),
        LBC_TYPE_THREAD => return c"thread".as_ptr(),
        LBC_TYPE_USERDATA => return c"userdata".as_ptr(),
        LBC_TYPE_VECTOR => return c"vector".as_ptr(),
        LBC_TYPE_BUFFER => return c"buffer".as_ptr(),
        LBC_TYPE_ANY => return c"any".as_ptr(),
        _ => {}
    }

    LUAU_ASSERT!(false);
    core::ptr::null()
}
