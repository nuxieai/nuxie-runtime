use luaur_ast::records::ast_name::AstName;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_table::DenseDefault;

use crate::enums::global::Global;

impl DenseDefault for Global {
    fn dense_default() -> Self {
        Global::Default
    }
}

#[inline]
pub(crate) fn get_global_state(globals: &DenseHashMap<AstName, Global>, name: AstName) -> Global {
    match globals.find(&name) {
        Some(&g) => g,
        None => Global::Default,
    }
}
