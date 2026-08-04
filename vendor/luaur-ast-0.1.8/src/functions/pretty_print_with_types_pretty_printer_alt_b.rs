use crate::functions::pretty_print_with_types_pretty_printer::pretty_print_with_types_ast_stat_block_cst_node_map;
use crate::records::ast_stat_block::AstStatBlock;
use crate::type_aliases::cst_node_map::CstNodeMap;
use luaur_common::records::dense_hash_map::DenseHashMap;

pub fn pretty_print_with_types_ast_stat_block(block: &mut AstStatBlock) -> alloc::string::String {
    let cst_node_map = CstNodeMap::new(core::ptr::null_mut());
    pretty_print_with_types_ast_stat_block_cst_node_map(block, cst_node_map)
}
