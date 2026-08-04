use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::ast_stat_repeat::AstStatRepeat;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatRepeat {
    pub fn new(
        location: Location,
        condition: *mut AstExpr,
        body: *mut AstStatBlock,
        DEPRECATED_hasUntil: bool,
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
            DEPRECATED_hasUntil,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_repeat_ast_stat_repeat(
    location: Location,
    condition: *mut AstExpr,
    body: *mut AstStatBlock,
    DEPRECATED_hasUntil: bool,
) -> AstStatRepeat {
    AstStatRepeat::new(location, condition, body, DEPRECATED_hasUntil)
}
