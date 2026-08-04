use crate::enums::type_constant_folding::Type;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn get_constant(&mut self, node: *mut AstExpr) -> Constant {
        if let Some(cv) = self.constants.find(&node) {
            *cv
        } else {
            Constant {
                r#type: Type::Type_Unknown,
                string_length: 0,
                data: unsafe { core::mem::zeroed() },
            }
        }
    }
}
