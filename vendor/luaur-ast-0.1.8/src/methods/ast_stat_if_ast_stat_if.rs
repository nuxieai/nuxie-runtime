use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::ast_stat_if::AstStatIf;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatIf {
    pub fn new(
        location: Location,
        condition: *mut AstExpr,
        thenbody: *mut AstStatBlock,
        elsebody: *mut AstStat,
        then_location: Option<Location>,
        else_location: Option<Location>,
    ) -> Self {
        Self {
            base: AstStat {
                base: AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            condition,
            thenbody,
            elsebody,
            then_location,
            else_location,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_if_ast_stat_if(
    location: Location,
    condition: *mut AstExpr,
    thenbody: *mut AstStatBlock,
    elsebody: *mut AstStat,
    then_location: Option<Location>,
    else_location: Option<Location>,
) -> AstStatIf {
    AstStatIf::new(
        location,
        condition,
        thenbody,
        elsebody,
        then_location,
        else_location,
    )
}
