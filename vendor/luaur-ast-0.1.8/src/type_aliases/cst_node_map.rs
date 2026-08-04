use crate::records::ast_node::AstNode;
use crate::records::cst_node::CstNode;
use luaur_common::records::dense_hash_map::DenseHashMap;

pub type CstNodeMap = DenseHashMap<*mut AstNode, *mut CstNode>;
