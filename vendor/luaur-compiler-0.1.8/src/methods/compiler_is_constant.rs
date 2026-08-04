use crate::enums::type_constant_folding::Type;
use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn is_constant(&mut self, node: *mut AstExpr) -> bool {
        if let Some(cv) = self.constants.find(&node) {
            return cv.r#type != Type::Type_Unknown;
        }
        false
    }
}
