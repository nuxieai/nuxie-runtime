use crate::records::ast_table_indexer::AstTableIndexer;
use crate::records::position::Position;

#[derive(Debug, Clone)]
pub struct TableIndexerResult {
    pub node: *mut AstTableIndexer,
    pub indexer_open_position: Position,
    pub indexer_close_position: Position,
    pub colon_position: Position,
}
