use alloc::string::String;

use crate::records::ast_node::AstNode;
use crate::records::printer::Printer;
use crate::records::string_writer::StringWriter;
use crate::records::writer::Writer;
use crate::type_aliases::cst_node_map::CstNodeMap;

pub fn to_string_ast_node(node: *mut AstNode) -> String {
    let node_ref = unsafe { &*node };

    let mut writer = StringWriter {
        ss: alloc::string::String::new(),
        pos: node_ref.location.begin,
        last_char: '\0',
    };

    let mut printer = Printer::new(
        &mut writer as &mut dyn Writer,
        CstNodeMap::new(core::ptr::null_mut()),
    );
    printer.write_types = true;

    let stat_node = node_ref.as_stat_const();
    if !stat_node.is_null() {
        let stat_node_mut = unsafe { &mut *(stat_node as *mut crate::records::ast_stat::AstStat) };
        printer.visualize_ast_stat(stat_node_mut);
    } else {
        let expr_node = node_ref.as_expr_const();
        if !expr_node.is_null() {
            let expr_node_mut =
                unsafe { &mut *(expr_node as *mut crate::records::ast_expr::AstExpr) };
            printer.visualize_ast_expr(expr_node_mut);
        } else {
            let type_node = unsafe { &mut *node_ref.as_type() };
            printer.visualize_type_annotation(type_node);
        }
    }

    writer.str().clone()
}
