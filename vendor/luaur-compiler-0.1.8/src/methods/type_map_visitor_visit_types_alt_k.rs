use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_unary::AstExprUnary;
use luaur_ast::records::ast_expr_unary::AstExprUnaryOp;
use luaur_ast::visit;
use luaur_common::enums::luau_bytecode_type::{LuauBytecodeType, LBC_TYPE_NUMBER, LBC_TYPE_VECTOR};

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_unary(&mut self, node: *mut AstExprUnary) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let node_ref = &*node;
            let expr = node_ref.expr;

            visit::ast_expr_visit(expr, self);

            match node_ref.op {
                AstExprUnaryOp::Not => {
                    self.record_resolved_type_ast_expr_ast_type(
                        node as *mut AstExpr,
                        &self.builtin_types.boolean_type as *const _
                            as *const luaur_ast::records::ast_type::AstType,
                    );
                }
                AstExprUnaryOp::Minus => {
                    let type_ptr = self.resolved_exprs.find(&expr);
                    let bc_type_ptr = self.expr_types.find(&expr);

                    if let (Some(&ty), Some(&bc_ty)) = (type_ptr, bc_type_ptr) {
                        if bc_ty == LBC_TYPE_VECTOR || bc_ty == LBC_TYPE_NUMBER {
                            self.record_resolved_type_ast_expr_ast_type(node as *mut AstExpr, ty);
                        }
                    }
                }
                AstExprUnaryOp::Len => {
                    self.record_resolved_type_ast_expr_ast_type(
                        node as *mut AstExpr,
                        &self.builtin_types.number_type as *const _
                            as *const luaur_ast::records::ast_type::AstType,
                    );
                }
            }

            false
        }
    }
}
