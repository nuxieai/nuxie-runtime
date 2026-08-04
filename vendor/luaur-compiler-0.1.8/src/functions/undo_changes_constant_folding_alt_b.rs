use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use crate::type_aliases::local_constant_change_log::LocalConstantChangeLog;
use luaur_ast::records::ast_local::AstLocal;
use luaur_common::records::dense_hash_map::DenseHashMap;

pub fn undo_changes_dense_hash_map_ast_local_constant_local_constant_change_log(
    locals: &mut DenseHashMap<*mut AstLocal, Constant>,
    changes: &LocalConstantChangeLog,
) {
    for it in changes.iter().rev() {
        if it.was_absent {
            if let Some(old) = locals.find(&it.key) {
                let _ = old;
            }
            if let Some(old) = locals.find_mut(&it.key) {
                old.r#type = Type::Type_Unknown;
            }
        } else {
            let old_value = it.old_value;
            *locals.get_or_insert(it.key) = old_value;
        }
    }
}
