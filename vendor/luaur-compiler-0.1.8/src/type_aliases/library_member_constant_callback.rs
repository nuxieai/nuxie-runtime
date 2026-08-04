use crate::type_aliases::compile_constant::CompileConstant;

pub type LibraryMemberConstantCallback = Option<
    unsafe extern "C" fn(
        library: *const core::ffi::c_char,
        member: *const core::ffi::c_char,
        constant: *mut CompileConstant,
    ),
>;
