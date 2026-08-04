use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

use crate::records::shape_visitor::ShapeVisitor;
use crate::records::table_shape::TableShape;

pub fn predict_table_shapes(
    shapes: &mut DenseHashMap<*mut AstExprTable, TableShape>,
    root: *mut AstNode,
) {
    let mut visitor = ShapeVisitor {
        shapes,
        tables: DenseHashMap::new(core::ptr::null_mut()),
        fields: DenseHashSet::new((core::ptr::null_mut(), AstName::default())),
        loops: DenseHashMap::new(core::ptr::null_mut()),
    };

    unsafe {
        luaur_ast::visit::ast_node_visit(root, &mut visitor);
    }
}
