use crate::records::ast_array::AstArray;
use crate::records::cst_attr_list::CstAttrList;
use crate::records::position::Position;

impl CstAttrList {
    pub fn new(
        at_bracket_position: Position,
        close_bracket_position: Position,
        comma_positions: AstArray<Position>,
    ) -> Self {
        luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauCstAttr.get());
        Self {
            at_bracket_position,
            close_bracket_position,
            comma_positions,
        }
    }
}
