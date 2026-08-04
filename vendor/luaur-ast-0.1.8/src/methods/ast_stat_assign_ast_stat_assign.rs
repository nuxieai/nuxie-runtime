use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_assign::AstStatAssign;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatAssign {
    pub fn new(
        location: Location,
        vars: AstArray<*mut AstExpr>,
        values: AstArray<*mut AstExpr>,
    ) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            vars,
            values,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_assign_ast_stat_assign(
    location: Location,
    vars: AstArray<*mut AstExpr>,
    values: AstArray<*mut AstExpr>,
) -> AstStatAssign {
    AstStatAssign::new(location, vars, values)
}
