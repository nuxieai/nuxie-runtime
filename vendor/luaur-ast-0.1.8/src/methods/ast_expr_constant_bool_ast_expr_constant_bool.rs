use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_constant_bool::AstExprConstantBool;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprConstantBool {
    pub fn new(location: Location, value: bool) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as crate::rtti::AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            value,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_constant_bool_ast_expr_constant_bool(
    location: Location,
    value: bool,
) -> AstExprConstantBool {
    AstExprConstantBool::new(location, value)
}
