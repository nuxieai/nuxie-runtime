use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::ast_stat_while::AstStatWhile;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatWhile {
    pub fn new(
        location: Location,
        condition: *mut AstExpr,
        body: *mut AstStatBlock,
        has_do: bool,
        do_location: Location,
    ) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as crate::rtti::AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            condition,
            body,
            has_do,
            do_location,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_while_ast_stat_while(
    location: Location,
    condition: *mut AstExpr,
    body: *mut AstStatBlock,
    has_do: bool,
    do_location: Location,
) -> AstStatWhile {
    AstStatWhile::new(location, condition, body, has_do, do_location)
}
