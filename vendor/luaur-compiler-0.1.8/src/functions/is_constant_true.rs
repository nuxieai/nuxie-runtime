use luaur_ast::records::ast_expr::AstExpr;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;

pub(crate) fn is_constant_true(
    constants: &DenseHashMap<*mut AstExpr, Constant>,
    node: *mut AstExpr,
) -> bool {
    match constants.find(&node) {
        Some(cv) if cv.r#type != Type::Type_Unknown => cv.is_truthful(),
        _ => false,
    }
}
