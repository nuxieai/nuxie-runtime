use crate::records::cst_expr_group::CstExprGroup;
use crate::records::position::Position;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl CstExprGroup {
    pub fn new(close_position: Position) -> Self {
        LUAU_ASSERT!(luaur_common::FFlag::LuauCstExprGroup.get());

        Self {
            base: crate::records::cst_node::CstNode {
                class_index: <Self as crate::rtti::CstNodeClass>::CLASS_INDEX,
            },
            close_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_expr_group_cst_expr_group(close_position: Position) -> CstExprGroup {
    CstExprGroup::new(close_position)
}
