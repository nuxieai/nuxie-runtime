use crate::records::ast_expr_if_else::AstExprIfElse;
use crate::records::cst_expr_if_else::CstExprIfElse;
use crate::records::printer::Printer;

impl<'a> Printer<'a> {
    pub fn visualize_else_if_expr(&mut self, elseif: &mut AstExprIfElse) {
        let cst_node = self.lookup_cst_node::<CstExprIfElse>(
            elseif as *mut AstExprIfElse as *mut crate::records::ast_node::AstNode,
        );

        self.visualize_ast_expr(unsafe { &mut *elseif.condition });

        if !cst_node.is_null() {
            unsafe {
                self.maybe_advance_and_write(&(*cst_node).then_position, "then", false);
            }
        } else {
            self.writer.keyword("then");
        }

        self.visualize_ast_expr(unsafe { &mut *elseif.true_expr });

        if elseif.has_else {
            if !cst_node.is_null() {
                unsafe { self.advance(&(*cst_node).else_position) };
            }

            let else_expr = unsafe { elseif.false_expr as *mut AstExprIfElse };
            if !else_expr.is_null() {
                let elseifelseif = unsafe {
                    crate::rtti::ast_node_as::<AstExprIfElse>(
                        else_expr as *mut crate::records::ast_node::AstNode,
                    )
                };
                if !elseifelseif.is_null()
                    && (cst_node.is_null() || unsafe { (*cst_node).is_else_if })
                {
                    self.writer.keyword("elseif");
                    self.visualize_else_if_expr(unsafe { &mut *elseifelseif });
                    return;
                }
            }

            self.writer.keyword("else");
            self.visualize_ast_expr(unsafe { &mut *elseif.false_expr });
        }
    }
}
