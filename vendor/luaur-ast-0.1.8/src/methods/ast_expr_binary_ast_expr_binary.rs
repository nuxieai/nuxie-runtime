use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_binary::{AstExprBinary, AstExprBinary_Op};
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprBinary {
    pub fn new(
        location: Location,
        op: AstExprBinary_Op,
        left: *mut AstExpr,
        right: *mut AstExpr,
    ) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            op,
            left,
            right,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_binary_ast_expr_binary(
    location: Location,
    op: AstExprBinary_Op,
    left: *mut AstExpr,
    right: *mut AstExpr,
) -> AstExprBinary {
    AstExprBinary::new(location, op, left, right)
}
