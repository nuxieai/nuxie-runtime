use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_unary::{AstExprUnary, AstExprUnaryOp};
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprUnary {
    pub fn new(location: Location, op: AstExprUnaryOp, expr: *mut AstExpr) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            op,
            expr,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_unary_ast_expr_unary(
    location: Location,
    op: AstExprUnaryOp,
    expr: *mut AstExpr,
) -> AstExprUnary {
    AstExprUnary::new(location, op, expr)
}
