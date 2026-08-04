use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_expr::AstStatExpr;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatExpr {
    pub fn new(location: Location, expr: *mut AstExpr) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            expr,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_expr_ast_stat_expr(location: Location, expr: *mut AstExpr) -> AstStatExpr {
    AstStatExpr::new(location, expr)
}
