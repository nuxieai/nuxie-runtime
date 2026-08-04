use crate::records::allocator::Allocator;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_index_expr::AstExprIndexExpr;
use crate::records::cst_expr_index_expr::CstExprIndexExpr;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use crate::records::position::Position;

impl Parser {
    pub(crate) fn parse_index_expr(&mut self, start: Position, expr: *mut AstExpr) -> *mut AstExpr {
        let match_bracket = MatchLexeme::new(self.lexer.current());
        self.next_lexeme();

        let index = self.parse_expr_i32(0);

        let close_bracket_position = self.lexer.current().location.begin;
        let end = self.lexer.current().location.end;

        let closing_bracket_found = self.expect_match_and_consume(']', &match_bracket, true);

        let expr = unsafe {
            (*self.allocator).alloc(AstExprIndexExpr::new(
                Location::new(start, end),
                expr,
                index,
            ))
        };

        if self.options.store_cst_data {
            let cst_node = unsafe {
                (*self.allocator).alloc(CstExprIndexExpr::new(
                    match_bracket.position,
                    if closing_bracket_found {
                        close_bracket_position
                    } else {
                        Position::missing()
                    },
                ))
            };
            self.cst_node_map.try_insert(
                expr as *mut crate::records::ast_node::AstNode,
                cst_node as *mut crate::records::cst_node::CstNode,
            );
        }

        expr as *mut AstExpr
    }
}
