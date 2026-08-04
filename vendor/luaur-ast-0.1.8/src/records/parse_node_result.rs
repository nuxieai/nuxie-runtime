use crate::records::ast_node::AstNode;
use crate::records::comment::Comment;
use crate::records::hot_comment::HotComment;
use crate::records::parse_error::ParseError;
use crate::type_aliases::cst_node_map::CstNodeMap;
use alloc::vec::Vec;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct ParseNodeResult<Node = AstNode> {
    pub root: *mut Node,
    pub lines: usize,
    pub hotcomments: Vec<HotComment>,
    pub errors: Vec<ParseError>,
    pub comment_locations: Vec<Comment>,
    pub cst_node_map: CstNodeMap,
}
