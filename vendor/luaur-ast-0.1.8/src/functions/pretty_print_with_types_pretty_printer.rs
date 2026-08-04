use crate::records::ast_stat_block::AstStatBlock;
use crate::records::printer::Printer;
use crate::records::string_writer::StringWriter;
use crate::type_aliases::cst_node_map::CstNodeMap;

pub fn pretty_print_with_types_ast_stat_block_cst_node_map(
    block: &mut AstStatBlock,
    cst_node_map: CstNodeMap,
) -> alloc::string::String {
    let mut writer = StringWriter {
        ss: alloc::string::String::new(),
        pos: crate::records::position::Position::default(),
        last_char: ' ',
    };
    let mut printer = Printer::new(&mut writer, cst_node_map);
    printer.write_types = true;
    printer.visualize_block_ast_stat_block(block);
    writer.str().clone()
}
