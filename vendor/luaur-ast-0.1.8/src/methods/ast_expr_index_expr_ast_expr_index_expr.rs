use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_index_expr::AstExprIndexExpr;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprIndexExpr {
    pub fn new(location: Location, expr: *mut AstExpr, index: *mut AstExpr) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            expr,
            index,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_index_expr_ast_expr_index_expr(
    location: Location,
    expr: *mut AstExpr,
    index: *mut AstExpr,
) -> AstExprIndexExpr {
    AstExprIndexExpr::new(location, expr, index)
}
