use crate::records::cst_node::CstNode;
use crate::records::cst_stat_do::CstStatDo;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstStatDo {
    pub fn new(stats_start_position: Position, end_position: Position) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            stats_start_position: stats_start_position,
            end_position: end_position,
        }
    }
}

#[allow(non_snake_case)]
pub fn cst_stat_do_cst_stat_do(
    stats_start_position: Position,
    end_position: Position,
) -> CstStatDo {
    CstStatDo::new(stats_start_position, end_position)
}
