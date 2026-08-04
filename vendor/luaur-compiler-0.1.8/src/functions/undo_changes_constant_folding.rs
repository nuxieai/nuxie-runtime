use crate::records::constant::Constant;
use crate::type_aliases::expr_constant_change_log::ExprConstantChangeLog;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_common::records::dense_hash_map::DenseHashMap;

pub fn undo_changes_dense_hash_map_ast_expr_constant_expr_constant_change_log(
    constants: &mut DenseHashMap<*mut AstExpr, Constant>,
    changes: &ExprConstantChangeLog,
) {
    for it in changes.iter().rev() {
        if it.was_absent {
            if let Some(old) = constants.find_mut(&it.key) {
                old.r#type = crate::enums::type_constant_folding::Type::Type_Unknown;
            }
        } else {
            let old = it.old_value;
            *constants.get_or_insert(it.key) = old;
        }
    }
}
