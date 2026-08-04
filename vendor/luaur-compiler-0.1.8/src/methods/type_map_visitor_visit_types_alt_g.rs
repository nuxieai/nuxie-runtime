use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_type::AstType;
use luaur_common::enums::luau_bytecode_type::{LuauBytecodeType, LBC_TYPE_ANY};

impl<'a> TypeMapVisitor<'a> {
    pub fn visit_ast_expr_local(&mut self, node: *mut AstExprLocal) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let local = (*node).local;
            if local.is_null() {
                return false;
            }

            if !(*local).annotation.is_null() {
                let annotation = (*local).annotation;
                let ty =
                    self.record_resolved_type_ast_expr_ast_type(node as *mut AstExpr, annotation);

                if ty != LBC_TYPE_ANY {
                    self.local_types.try_insert(local, ty);
                }
            } else if let Some(type_ptr) = self.resolved_locals.find(&local) {
                let ty =
                    self.record_resolved_type_ast_expr_ast_type(node as *mut AstExpr, *type_ptr);
                self.local_types.try_insert(local, ty);
            }

            false
        }
    }
}
