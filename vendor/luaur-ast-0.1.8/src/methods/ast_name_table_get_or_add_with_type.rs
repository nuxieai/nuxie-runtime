//! `std::pair<AstName, Lexeme::Type> AstNameTable::get_or_add_with_type(const char* name, size_t length)`
//! — Ast/src/Lexer.cpp:224.

use crate::records::allocator::Allocator;
use crate::records::ast_name::AstName;
use crate::records::ast_name_table::AstNameTable;
use crate::records::entry::Entry;
use crate::records::lexeme::Type;

impl AstNameTable {
    pub fn get_or_add_with_type(
        &mut self,
        name: *const core::ffi::c_char,
        length: usize,
    ) -> (AstName, Type) {
        // Read the allocator pointer out before borrowing `self.data`, so the
        // in-place fixup below doesn't conflict with the `insert_mut` borrow.
        let allocator: *mut Allocator = self.allocator;

        let key = Entry {
            value: AstName { value: name },
            length: length as u32,
            r#type: Type::Eof,
        };

        let entry = self.data.insert_mut(key);

        // entry already was inserted
        if entry.r#type != Type::Eof {
            return (entry.value, entry.r#type);
        }

        // We just inserted an entry with a non-owned pointer into the map; we
        // need to correct it, *but* must not disturb the hash value, so we copy
        // the same bytes into an allocator-owned buffer and repoint at it.
        // (C++ does this via `const_cast<Entry&>`; `insert_mut` is the faithful
        // Rust spelling of that mutation.)
        let name_data = unsafe { (*allocator).allocate(length + 1) };
        unsafe {
            core::ptr::copy_nonoverlapping(name as *const u8, name_data, length);
            *name_data.add(length) = 0;
        }

        let first = unsafe { *name };

        entry.value = AstName {
            value: name_data as *const core::ffi::c_char,
        };
        entry.r#type = if first == b'@' as core::ffi::c_char {
            Type::Attribute
        } else {
            Type::Name
        };

        (entry.value, entry.r#type)
    }
}
