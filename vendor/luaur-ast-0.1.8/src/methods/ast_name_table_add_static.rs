use crate::records::ast_name::AstName;
use crate::records::ast_name_table::AstNameTable;
use crate::records::entry::Entry;
use crate::records::lexeme::Type;

impl AstNameTable {
    pub fn add_static(&mut self, name: *const core::ffi::c_char, r#type: Type) -> AstName {
        let length = unsafe { core::ffi::CStr::from_ptr(name).to_bytes().len() as u32 };
        let entry = Entry {
            value: AstName { value: name },
            length,
            r#type,
        };

        luaur_common::LUAU_ASSERT!(!self.data.contains(&entry));
        self.data.insert(entry);

        entry.value
    }
}
