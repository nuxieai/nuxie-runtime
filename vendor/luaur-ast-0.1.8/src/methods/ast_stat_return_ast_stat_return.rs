use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_return::AstStatReturn;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatReturn {
    pub fn new(location: Location, list: AstArray<*mut AstExpr>) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            list,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_return_ast_stat_return(
    location: Location,
    list: AstArray<*mut AstExpr>,
) -> AstStatReturn {
    AstStatReturn::new(location, list)
}
