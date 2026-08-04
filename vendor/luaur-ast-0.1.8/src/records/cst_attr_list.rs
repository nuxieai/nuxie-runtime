use crate::records::ast_array::AstArray;
use crate::records::position::Position;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CstAttrList {
    pub at_bracket_position: Position,
    pub close_bracket_position: Position,
    pub comma_positions: AstArray<Position>,
}
