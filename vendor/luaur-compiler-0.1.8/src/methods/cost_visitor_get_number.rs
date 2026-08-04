use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use crate::records::cost_visitor::CostVisitor;
use luaur_ast::records::ast_expr::AstExpr;

impl CostVisitor {
    pub fn get_number(&self, node: *mut AstExpr, result: &mut f64) -> bool {
        unsafe {
            if let Some(constant) = (*self.constants).find(&node) {
                if constant.r#type == Type::Type_Number {
                    *result = constant.data.value_number;
                    return true;
                }
            }
        }
        false
    }
}
