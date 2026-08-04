use luaur_ast::records::ast_expr::AstExpr;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;

#[inline]
pub fn is_constant_false(
    constants: &DenseHashMap<*mut AstExpr, Constant>,
    node: *mut AstExpr,
) -> bool {
    match constants.find(&node) {
        Some(cv) => cv.r#type != Type::Type_Unknown && !cv.is_truthful(),
        None => false,
    }
}
