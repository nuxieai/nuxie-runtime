use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_stat_local::CstStatLocal;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstStatLocal {
    pub fn new(
        vars_annotation_colon_positions: AstArray<Position>,
        vars_comma_positions: AstArray<Position>,
        values_comma_positions: AstArray<Position>,
    ) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            declaration_keyword_position: Position::missing(),
            vars_annotation_colon_positions: vars_annotation_colon_positions,
            vars_comma_positions: vars_comma_positions,
            values_comma_positions: values_comma_positions,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_stat_local_cst_stat_local(
    vars_annotation_colon_positions: AstArray<Position>,
    vars_comma_positions: AstArray<Position>,
    values_comma_positions: AstArray<Position>,
) -> CstStatLocal {
    CstStatLocal::new(
        vars_annotation_colon_positions,
        vars_comma_positions,
        values_comma_positions,
    )
}
