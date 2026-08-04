use crate::records::ast_stat_block::AstStatBlock;
use crate::records::comment::Comment;
use crate::records::hot_comment::HotComment;
use crate::records::parse_error::ParseError;
use crate::type_aliases::cst_node_map::CstNodeMap;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub root: *mut AstStatBlock,
    pub lines: usize,
    pub hotcomments: Vec<HotComment>,
    pub errors: Vec<ParseError>,
    pub comment_locations: Vec<Comment>,
    pub cst_node_map: CstNodeMap,
}

impl Default for ParseResult {
    fn default() -> Self {
        Self {
            root: core::ptr::null_mut(),
            lines: 0,
            hotcomments: Vec::new(),
            errors: Vec::new(),
            comment_locations: Vec::new(),
            cst_node_map: CstNodeMap::new(core::ptr::null_mut()),
        }
    }
}
