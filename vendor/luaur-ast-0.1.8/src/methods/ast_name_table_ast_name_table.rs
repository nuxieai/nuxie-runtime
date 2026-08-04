use crate::records::allocator::Allocator;
use crate::records::ast_name_table::AstNameTable;
use crate::records::entry::Entry;
use crate::records::lexeme::Type;
use luaur_common::records::dense_hash_set::DenseHashSet;

#[allow(non_upper_case_globals)]
const kReserved: [*const core::ffi::c_char; 21] = [
    b"and\0".as_ptr() as *const core::ffi::c_char,
    b"break\0".as_ptr() as *const core::ffi::c_char,
    b"do\0".as_ptr() as *const core::ffi::c_char,
    b"else\0".as_ptr() as *const core::ffi::c_char,
    b"elseif\0".as_ptr() as *const core::ffi::c_char,
    b"end\0".as_ptr() as *const core::ffi::c_char,
    b"false\0".as_ptr() as *const core::ffi::c_char,
    b"for\0".as_ptr() as *const core::ffi::c_char,
    b"function\0".as_ptr() as *const core::ffi::c_char,
    b"if\0".as_ptr() as *const core::ffi::c_char,
    b"in\0".as_ptr() as *const core::ffi::c_char,
    b"local\0".as_ptr() as *const core::ffi::c_char,
    b"nil\0".as_ptr() as *const core::ffi::c_char,
    b"not\0".as_ptr() as *const core::ffi::c_char,
    b"or\0".as_ptr() as *const core::ffi::c_char,
    b"repeat\0".as_ptr() as *const core::ffi::c_char,
    b"return\0".as_ptr() as *const core::ffi::c_char,
    b"then\0".as_ptr() as *const core::ffi::c_char,
    b"true\0".as_ptr() as *const core::ffi::c_char,
    b"until\0".as_ptr() as *const core::ffi::c_char,
    b"while\0".as_ptr() as *const core::ffi::c_char,
];

impl AstNameTable {
    pub fn new(allocator: &mut Allocator) -> Self {
        let mut table = Self {
            // C++ pre-sizes the set to 128 buckets; DenseHashSet::new grows on
            // demand from the empty sentinel (a non-observable difference).
            data: DenseHashSet::new(Entry {
                value: crate::records::ast_name::AstName {
                    value: core::ptr::null(),
                },
                length: 0,
                r#type: Type::Eof,
            }),
            allocator: allocator as *mut Allocator,
        };

        for i in (Type::Reserved_BEGIN.0)..(Type::Reserved_END.0) {
            let index = (i - Type::Reserved_BEGIN.0) as usize;
            table.add_static(kReserved[index], Type(i));
        }

        table
    }
}

#[allow(non_snake_case)]
pub fn ast_name_table_ast_name_table(allocator: &mut Allocator) -> AstNameTable {
    AstNameTable::new(allocator)
}
