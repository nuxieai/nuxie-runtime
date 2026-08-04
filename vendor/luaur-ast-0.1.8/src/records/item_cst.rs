use crate::enums::separator::Separator;
use crate::records::position::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Item {
    pub indexer_open_position: Position,
    pub indexer_close_position: Position,
    pub equals_position: Position,
    pub separator: Separator,
    pub separator_position: Position,
}
