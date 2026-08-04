use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_group::AstExprGroup;
use crate::records::ast_node::AstNode;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstExprGroup {
    pub fn new(location: Location, expr: *mut AstExpr) -> Self {
        Self {
            base: AstExpr {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
            },
            expr,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_group_ast_expr_group(location: Location, expr: *mut AstExpr) -> AstExprGroup {
    AstExprGroup::new(location, expr)
}
