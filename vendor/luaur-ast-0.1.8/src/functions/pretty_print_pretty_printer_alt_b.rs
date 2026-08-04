use crate::functions::pretty_print_pretty_printer::pretty_print_ast_stat_block_cst_node_map;
use crate::records::ast_stat_block::AstStatBlock;
use crate::type_aliases::cst_node_map::CstNodeMap;

pub fn pretty_print_ast_stat_block(block: &mut AstStatBlock) -> alloc::string::String {
    pretty_print_ast_stat_block_cst_node_map(block, CstNodeMap::new(core::ptr::null_mut()))
}
