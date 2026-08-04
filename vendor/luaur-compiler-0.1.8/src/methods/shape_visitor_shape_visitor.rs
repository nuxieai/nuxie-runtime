use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_name::AstName;

use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;
use luaur_common::records::dense_hash_table::DenseHasher;

use crate::records::shape_visitor::ShapeVisitor;
use crate::records::hasher::Hasher;

impl DenseHasher<(*mut AstExprTable, AstName)> for Hasher {
    fn hash(&self, key: &(*mut AstExprTable, AstName)) -> usize {
        crate::methods::shape_visitor_hasher_operator_call::shape_visitor_hasher_operator_call(*key)
    }
}

pub fn shape_visitor_shape_visitor(
    shapes: &mut DenseHashMap<*mut AstExprTable, crate::records::table_shape::TableShape>,
) -> ShapeVisitor<'_> {
    ShapeVisitor {
        shapes,
        tables: DenseHashMap::new(core::ptr::null_mut()),
        fields: DenseHashSet::new((core::ptr::null_mut(), AstName::new())),
        loops: DenseHashMap::new(core::ptr::null_mut()),
    }
}
