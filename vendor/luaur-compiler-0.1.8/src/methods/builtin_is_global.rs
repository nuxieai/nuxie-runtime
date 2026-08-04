use crate::records::builtin::Builtin;
use core::ffi::CStr;
use luaur_ast::records::ast_name::AstName;

impl Builtin {
    pub fn is_global(&self, name: &str) -> bool {
        if self.object != AstName::default() || self.method.value.is_null() {
            return false;
        }
        unsafe {
            let method_bytes = CStr::from_ptr(self.method.value).to_bytes();
            method_bytes == name.as_bytes()
        }
    }
}
