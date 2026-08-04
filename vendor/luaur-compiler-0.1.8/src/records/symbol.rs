use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol {
    pub local: *mut AstLocal,
    pub global: AstName,
}

impl Symbol {
    pub fn new() -> Self {
        Self {
            local: core::ptr::null_mut(),
            global: AstName::new(),
        }
    }

    pub fn from_local(local: *mut AstLocal) -> Self {
        Self {
            local,
            global: AstName::new(),
        }
    }

    pub fn from_global(global: AstName) -> Self {
        Self {
            local: core::ptr::null_mut(),
            global,
        }
    }

    pub fn ast_name(&self) -> AstName {
        if !self.local.is_null() {
            unsafe { (*self.local).name }
        } else {
            luaur_common::LUAU_ASSERT!(!self.global.value.is_null());
            self.global
        }
    }

    pub fn c_str(&self) -> *const core::ffi::c_char {
        if !self.local.is_null() {
            unsafe { (*self.local).name.value }
        } else {
            luaur_common::LUAU_ASSERT!(!self.global.value.is_null());
            self.global.value
        }
    }
}

impl Default for Symbol {
    fn default() -> Self {
        Self::new()
    }
}

impl luaur_common::records::dense_hash_table::DenseDefault for Symbol {
    fn dense_default() -> Self {
        Self::default()
    }
}
