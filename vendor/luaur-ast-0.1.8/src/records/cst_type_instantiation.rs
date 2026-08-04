use crate::records::ast_array::AstArray;
use crate::records::position::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CstTypeInstantiation {
    pub left_arrow_1_position: Position,
    pub left_arrow_2_position: Position,

    pub comma_positions: AstArray<Position>,

    pub right_arrow_1_position: Position,
    pub right_arrow_2_position: Position,
}

impl Default for CstTypeInstantiation {
    fn default() -> Self {
        Self {
            left_arrow_1_position: Position {
                line: u32::MAX,
                column: u32::MAX,
            },
            left_arrow_2_position: Position {
                line: u32::MAX,
                column: u32::MAX,
            },
            comma_positions: AstArray {
                data: std::ptr::null_mut(),
                size: 0,
            },
            right_arrow_1_position: Position {
                line: u32::MAX,
                column: u32::MAX,
            },
            right_arrow_2_position: Position {
                line: u32::MAX,
                column: u32::MAX,
            },
        }
    }
}
