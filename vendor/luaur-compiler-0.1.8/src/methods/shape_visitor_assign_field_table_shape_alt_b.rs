use crate::records::shape_visitor::ShapeVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_constant_number::AstExprConstantNumber;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::rtti::ast_node_as;

impl<'a> ShapeVisitor<'a> {
    pub fn assign_field_ast_expr_ast_expr(&mut self, expr: *mut AstExpr, index: *mut AstExpr) {
        if expr.is_null() || index.is_null() {
            return;
        }

        let lv = unsafe { ast_node_as::<AstExprLocal>(expr as *mut AstNode) };
        if lv.is_null() {
            return;
        }

        let local = unsafe { (*lv).local };
        let table = self.tables.find(&local);
        if table.is_none() {
            return;
        }
        let table_ptr = *table.unwrap();

        let number = unsafe { ast_node_as::<AstExprConstantNumber>(index as *mut AstNode) };
        if !number.is_null() {
            let shape = self.shapes.get_or_insert(table_ptr);
            if unsafe { (*number).value } == (shape.array_size + 1) as f64 {
                shape.array_size += 1;
            }
        } else {
            let iter = unsafe { ast_node_as::<AstExprLocal>(index as *mut AstNode) };
            if !iter.is_null() {
                let iter_local = unsafe { (*iter).local };
                if let Some(&bound) = self.loops.find(&iter_local) {
                    let shape = self.shapes.get_or_insert(table_ptr);
                    if shape.array_size == 0 {
                        shape.array_size = bound;
                    }
                }
            }
        }
    }
}
