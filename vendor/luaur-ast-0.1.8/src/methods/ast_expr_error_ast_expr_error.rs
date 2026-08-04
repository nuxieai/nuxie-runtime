use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_error::AstExprError;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprError {
    pub fn new(
        location: Location,
        expressions: AstArray<*mut AstExpr>,
        message_index: u32,
    ) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            expressions,
            message_index,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_error_ast_expr_error(
    location: Location,
    expressions: AstArray<*mut AstExpr>,
    message_index: u32,
) -> AstExprError {
    AstExprError::new(location, expressions, message_index)
}
