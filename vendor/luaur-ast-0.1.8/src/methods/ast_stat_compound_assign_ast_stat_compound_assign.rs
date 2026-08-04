use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_binary::AstExprBinary_Op;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_compound_assign::AstStatCompoundAssign;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatCompoundAssign {
    pub fn new(
        location: Location,
        op: AstExprBinary_Op,
        var: *mut AstExpr,
        value: *mut AstExpr,
    ) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            op,
            var,
            value,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_compound_assign_ast_stat_compound_assign(
    location: Location,
    op: AstExprBinary_Op,
    var: *mut AstExpr,
    value: *mut AstExpr,
) -> AstStatCompoundAssign {
    AstStatCompoundAssign::new(location, op, var, value)
}
