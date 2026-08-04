use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_function::AstStatFunction;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatFunction {
    pub fn new(location: Location, name: *mut AstExpr, func: *mut AstExprFunction) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            name,
            func,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_function_ast_stat_function(
    location: Location,
    name: *mut AstExpr,
    func: *mut AstExprFunction,
) -> AstStatFunction {
    AstStatFunction::new(location, name, func)
}
