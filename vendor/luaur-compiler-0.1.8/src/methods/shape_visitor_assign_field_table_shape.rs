use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti::ast_node_as;

use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

use crate::records::shape_visitor::ShapeVisitor;
use crate::records::hasher::Hasher;

impl<'a> ShapeVisitor<'a> {
    pub fn assign_field_ast_expr_ast_name(&mut self, expr: *mut AstExpr, index: AstName) {
        if expr.is_null() {
            return;
        }

        let expr_ptr = expr as *mut AstNode;
        let local_expr = unsafe { ast_node_as::<AstExprLocal>(expr_ptr) };

        if !local_expr.is_null() {
            let local = unsafe { (*local_expr).local };
            if let Some(&table) = self.tables.find(&local) {
                let field = (table, index);

                if !self.fields.contains(&field) {
                    self.fields.insert(field);
                    self.shapes.get_or_insert(table).hash_size += 1;
                }
            }
        }
    }
}
