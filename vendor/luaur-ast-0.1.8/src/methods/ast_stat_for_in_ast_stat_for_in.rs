use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_local::AstLocal;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::ast_stat_for_in::AstStatForIn;
use crate::records::location::Location;
use crate::rtti::AstNodeClass;

impl AstStatForIn {
    pub fn new(
        location: Location,
        vars: AstArray<*mut AstLocal>,
        values: AstArray<*mut AstExpr>,
        body: *mut AstStatBlock,
        has_in: bool,
        in_location: Location,
        has_do: bool,
        do_location: Location,
    ) -> Self {
        Self {
            base: AstStat {
                base: crate::records::ast_node::AstNode {
                    class_index: <Self as AstNodeClass>::CLASS_INDEX,
                    location,
                },
                has_semicolon: false,
            },
            vars,
            values,
            body,
            has_in,
            in_location,
            has_do,
            do_location,
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_stat_for_in_ast_stat_for_in(
    location: Location,
    vars: AstArray<*mut AstLocal>,
    values: AstArray<*mut AstExpr>,
    body: *mut AstStatBlock,
    has_in: bool,
    in_location: Location,
    has_do: bool,
    do_location: Location,
) -> AstStatForIn {
    AstStatForIn::new(
        location,
        vars,
        values,
        body,
        has_in,
        in_location,
        has_do,
        do_location,
    )
}
