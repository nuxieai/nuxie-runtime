use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_error::AstStatError;
use crate::records::location::Location;

impl AstStatError {
    pub fn new(
        location: Location,
        expressions: AstArray<*mut AstExpr>,
        statements: AstArray<*mut AstStat>,
        message_index: u32,
    ) -> Self {
        AstStatError {
            base: AstStat {
                base: crate::records::ast_node::AstNode {
                    class_index: <Self as crate::rtti::AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            expressions,
            statements,
            message_index,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_error_ast_stat_error(
    location: Location,
    expressions: AstArray<*mut AstExpr>,
    statements: AstArray<*mut AstStat>,
    message_index: u32,
) -> AstStatError {
    AstStatError::new(location, expressions, statements, message_index)
}
