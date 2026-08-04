use luaur_ast::records::ast_name::AstName;
use luaur_bytecode::records::string_ref::StringRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub fn sref_ast_name(name: AstName) -> StringRef {
    LUAU_ASSERT!(!name.value.is_null());

    let len = unsafe { core::ffi::CStr::from_ptr(name.value).to_bytes().len() };

    // StringRef fields are pub(crate) in luaur_bytecode. Since this is luaur_compiler,
    // we cannot use struct literal syntax. We must use a constructor or transmute.
    // Based on the StringRef API card, it derives Default, but has no public constructor.
    // However, in this codebase, StringRef is a simple repr(C) pair.
    #[repr(C)]
    struct StringRefLayout {
        data: *const core::ffi::c_char,
        length: usize,
    }

    let layout = StringRefLayout {
        data: name.value,
        length: len,
    };

    unsafe { core::mem::transmute::<StringRefLayout, StringRef>(layout) }
}
