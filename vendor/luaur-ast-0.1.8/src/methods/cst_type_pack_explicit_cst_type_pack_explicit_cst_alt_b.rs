use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_type_pack_explicit::CstTypePackExplicit;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstTypePackExplicit {
    pub fn cst_type_pack_explicit_position_position_ast_array_position(
        open_parentheses_position: Position,
        close_parentheses_position: Position,
        comma_positions: AstArray<Position>,
    ) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            open_parentheses_position,
            close_parentheses_position,
            comma_positions,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_type_pack_explicit_position_position_ast_array_position(
    open_parentheses_position: Position,
    close_parentheses_position: Position,
    comma_positions: AstArray<Position>,
) -> CstTypePackExplicit {
    CstTypePackExplicit::cst_type_pack_explicit_position_position_ast_array_position(
        open_parentheses_position,
        close_parentheses_position,
        comma_positions,
    )
}
