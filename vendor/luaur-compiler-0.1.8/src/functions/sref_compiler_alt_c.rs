use core::ffi::c_char;
use luaur_ast::records::ast_array::AstArray;
use luaur_bytecode::records::string_ref::StringRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub fn sref_ast_array_c_char(data: AstArray<c_char>) -> StringRef {
    LUAU_ASSERT!(!data.begin().is_null());

    #[repr(C)]
    struct StringRefLayout {
        data: *const i8,
        length: usize,
    }

    let layout = StringRefLayout {
        data: data.begin() as *const i8,
        length: data.len(),
    };

    unsafe { core::mem::transmute::<StringRefLayout, StringRef>(layout) }
}
