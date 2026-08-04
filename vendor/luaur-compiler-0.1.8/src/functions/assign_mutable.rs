use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_name_table::AstNameTable;

use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::global::Global;

#[inline]
pub fn assign_mutable(
    globals: &mut DenseHashMap<AstName, Global>,
    names: &AstNameTable,
    mutable_globals: *const *const core::ffi::c_char,
) {
    let name = names.get(c"_G".as_ptr());
    if !name.value.is_null() {
        *globals.get_or_insert(name) = Global::Mutable;
    }

    if mutable_globals.is_null() {
        return;
    }

    let mut ptr = mutable_globals;
    unsafe {
        while !(*ptr).is_null() {
            let name = names.get(*ptr);
            if !name.value.is_null() {
                *globals.get_or_insert(name) = Global::Mutable;
            }
            ptr = ptr.add(1);
        }
    }
}
