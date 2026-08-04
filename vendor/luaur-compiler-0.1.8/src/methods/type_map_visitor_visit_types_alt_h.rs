use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_stat_local::AstStatLocal;
use luaur_ast::records::ast_type::AstType;

pub fn visit_ast_stat_local(this: &mut TypeMapVisitor<'_>, node: *mut AstStatLocal) -> bool {
    unsafe {
        if node.is_null() {
            return false;
        }

        let node_ref = &*node;

        // Visit all value expressions
        for &expr_ptr in node_ref.values.as_slice() {
            if !expr_ptr.is_null() {
                luaur_ast::visit::ast_expr_visit(expr_ptr, this);
            }
        }

        // Propagate types from values to variables
        let vars = node_ref.vars.as_slice();
        let values = node_ref.values.as_slice();

        for i in 0..vars.len() {
            let var_ptr = vars[i];
            if var_ptr.is_null() {
                continue;
            }

            let var = &mut *var_ptr;

            // Propagate from the value that's being assigned
            // This simple propagation doesn't handle type packs in tail position
            if var.annotation.is_null() {
                if i < values.len() {
                    let value_ptr = values[i];
                    if !value_ptr.is_null() {
                        if let Some(&type_ptr) = this.resolved_exprs.find(&value_ptr) {
                            this.resolved_locals
                                .try_insert(var_ptr as *mut AstLocal, type_ptr);
                        }
                    }
                }
            }
        }
    }

    false
}

impl<'a> TypeMapVisitor<'a> {
    pub fn visit_ast_stat_local(&mut self, node: *mut AstStatLocal) -> bool {
        visit_ast_stat_local(self, node)
    }
}

trait AstExprVisit {
    fn visit(&mut self, visitor: &mut TypeMapVisitor<'_>);
}

impl AstExprVisit for AstExpr {
    fn visit(&mut self, visitor: &mut TypeMapVisitor<'_>) {
        unsafe {
            luaur_ast::visit::ast_expr_visit(self as *mut AstExpr, visitor);
        }
    }
}
