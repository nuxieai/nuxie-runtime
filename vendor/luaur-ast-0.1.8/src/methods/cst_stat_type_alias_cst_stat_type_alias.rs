use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_stat_type_alias::CstStatTypeAlias;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstStatTypeAlias {
    pub fn new(
        type_keyword_position: Position,
        generics_open_position: Position,
        generics_comma_positions: AstArray<Position>,
        generics_close_position: Position,
        equals_position: Position,
    ) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            type_keyword_position,
            generics_open_position,
            generics_comma_positions,
            generics_close_position,
            equals_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_stat_type_alias_cst_stat_type_alias(
    type_keyword_position: Position,
    generics_open_position: Position,
    generics_comma_positions: AstArray<Position>,
    generics_close_position: Position,
    equals_position: Position,
) -> CstStatTypeAlias {
    CstStatTypeAlias::new(
        type_keyword_position,
        generics_open_position,
        generics_comma_positions,
        generics_close_position,
        equals_position,
    )
}
