use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr_binary::AstExprBinary;
use luaur_ast::records::ast_expr_binary::AstExprBinaryOp;
use luaur_ast::records::ast_type::AstType;
use luaur_common::enums::luau_bytecode_type::{LuauBytecodeType, LBC_TYPE_NUMBER, LBC_TYPE_VECTOR};

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_binary(&mut self, node: *mut AstExprBinary) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let left = (*node).left;
            let right = (*node).right;

            luaur_ast::visit::ast_expr_visit(
                left as *mut luaur_ast::records::ast_expr::AstExpr,
                self,
            );
            luaur_ast::visit::ast_expr_visit(
                right as *mut luaur_ast::records::ast_expr::AstExpr,
                self,
            );

            let op = (*node).op;

            if op == AstExprBinaryOp::CompareNe
                || op == AstExprBinaryOp::CompareEq
                || op == AstExprBinaryOp::CompareLt
                || op == AstExprBinaryOp::CompareLe
                || op == AstExprBinaryOp::CompareGt
                || op == AstExprBinaryOp::CompareGe
            {
                self.record_resolved_type_ast_expr_ast_type(
                    node as *mut luaur_ast::records::ast_expr::AstExpr,
                    &self.builtin_types.boolean_type as *const _ as *const AstType,
                );
                return false;
            }

            if op == AstExprBinaryOp::Concat
                || op == AstExprBinaryOp::And
                || op == AstExprBinaryOp::Or
            {
                return false;
            }

            let left_type_ptr = self
                .resolved_exprs
                .find(&(left as *mut luaur_ast::records::ast_expr::AstExpr));
            let left_bc_type_ptr = self
                .expr_types
                .find(&(left as *mut luaur_ast::records::ast_expr::AstExpr));

            if left_type_ptr.is_none() || left_bc_type_ptr.is_none() {
                return false;
            }

            let right_type_ptr = self
                .resolved_exprs
                .find(&(right as *mut luaur_ast::records::ast_expr::AstExpr));
            let right_bc_type_ptr = self
                .expr_types
                .find(&(right as *mut luaur_ast::records::ast_expr::AstExpr));

            if right_type_ptr.is_none() || right_bc_type_ptr.is_none() {
                return false;
            }

            let left_bc_type = *left_bc_type_ptr.unwrap();
            let right_bc_type = *right_bc_type_ptr.unwrap();

            if left_bc_type == LBC_TYPE_VECTOR {
                self.record_resolved_type_ast_expr_ast_type(
                    node as *mut luaur_ast::records::ast_expr::AstExpr,
                    *left_type_ptr.unwrap(),
                );
            } else if right_bc_type == LBC_TYPE_VECTOR {
                self.record_resolved_type_ast_expr_ast_type(
                    node as *mut luaur_ast::records::ast_expr::AstExpr,
                    *right_type_ptr.unwrap(),
                );
            } else if left_bc_type == LBC_TYPE_NUMBER && right_bc_type == LBC_TYPE_NUMBER {
                self.record_resolved_type_ast_expr_ast_type(
                    node as *mut luaur_ast::records::ast_expr::AstExpr,
                    *left_type_ptr.unwrap(),
                );
            }

            false
        }
    }
}
