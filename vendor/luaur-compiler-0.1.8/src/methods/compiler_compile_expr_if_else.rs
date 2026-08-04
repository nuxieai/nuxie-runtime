use crate::records::compiler::Compiler;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_if_else::AstExprIfElse;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl Compiler {
    pub fn compile_expr_if_else(
        &mut self,
        expr: *mut AstExprIfElse,
        target: u8,
        target_temp: bool,
    ) {
        let expr_ref = unsafe { &*expr };

        if self.is_constant(expr_ref.condition) {
            if self.is_constant_true(expr_ref.condition) {
                self.compile_expr(expr_ref.true_expr, target, target_temp);
            } else {
                self.compile_expr(expr_ref.false_expr, target, target_temp);
            }
        } else {
            let creg = self.get_expr_local_reg(expr_ref.condition);
            if creg >= 0 {
                let true_reg = self.get_expr_local_reg(expr_ref.true_expr);
                let false_reg = self.get_expr_local_reg(expr_ref.false_expr);

                if creg == true_reg && (false_reg >= 0 || self.is_constant(expr_ref.false_expr)) {
                    return self.compile_expr_if_else_and_or(
                        false,
                        creg as u8,
                        expr_ref.false_expr,
                        target,
                    );
                } else if creg == false_reg
                    && (true_reg >= 0 || self.is_constant(expr_ref.true_expr))
                {
                    return self.compile_expr_if_else_and_or(
                        true,
                        creg as u8,
                        expr_ref.true_expr,
                        target,
                    );
                }
            }

            let mut else_jump = Vec::new();
            self.compile_condition_value(
                expr_ref.condition,
                core::ptr::null(),
                &mut else_jump,
                false,
            );
            self.compile_expr(expr_ref.true_expr, target, target_temp);

            let bytecode = unsafe { &mut *self.bytecode };
            let then_label = bytecode.emit_label();
            bytecode.emit_ad(LuauOpcode::LOP_JUMP, 0, 0);

            let else_label = bytecode.emit_label();
            self.compile_expr(expr_ref.false_expr, target, target_temp);
            let end_label = bytecode.emit_label();

            self.patch_jumps(expr as *mut AstNode, &mut else_jump, else_label);
            self.patch_jump(expr as *mut AstNode, then_label, end_label);
        }
    }
}
