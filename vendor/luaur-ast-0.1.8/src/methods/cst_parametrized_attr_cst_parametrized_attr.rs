use crate::records::ast_array::AstArray;
use crate::records::cst_node::CstNode;
use crate::records::cst_parametrized_attr::CstParametrizedAttr;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstParametrizedAttr {
    pub fn new(
        open_paren_position: Position,
        close_paren_position: Position,
        args_comma_positions: AstArray<Position>,
    ) -> Self {
        luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauCstAttr.get());
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            open_paren_position,
            close_paren_position,
            args_comma_positions,
        }
    }
}
