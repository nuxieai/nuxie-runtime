use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_type_assertion::AstExprTypeAssertion;
use crate::records::cst_expr_type_assertion::CstExprTypeAssertion;
use crate::records::lexeme::Lexeme;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_assertion_expr(&mut self) -> *mut AstExpr {
        let start = self.lexer.current().location;
        let expr = self.parse_simple_expr();

        if self.lexer.current().r#type == crate::records::lexeme::Type::DoubleColon {
            let op_position = self.lexer.current().location.begin;
            self.next_lexeme();
            let annotation = self.parse_type_bool(false);

            let node = unsafe {
                (*self.allocator).alloc(AstExprTypeAssertion::new(
                    Location::new(start.begin, (*annotation).base.location.end),
                    expr,
                    annotation,
                ))
            };

            if self.options.store_cst_data {
                let cst_node =
                    unsafe { (*self.allocator).alloc(CstExprTypeAssertion::new(op_position)) };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }

            node as *mut AstExpr
        } else {
            expr
        }
    }
}
