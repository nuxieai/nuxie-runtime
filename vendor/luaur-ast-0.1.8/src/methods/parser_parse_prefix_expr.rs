use crate::enums::type_lexer::Type;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_group::AstExprGroup;
use crate::records::cst_expr_group::CstExprGroup;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use crate::records::position::Position;

impl Parser {
    pub fn parse_prefix_expr(&mut self) -> *mut AstExpr {
        if self.lexer.current().r#type == Type('(' as i32) {
            let start = self.lexer.current().location.begin;
            let match_paren = MatchLexeme::new(self.lexer.current());
            self.next_lexeme();

            let expr = self.parse_expr_i32(0);
            let mut end = self.lexer.current().location.end;
            let mut close_paren_found = false;

            if self.lexer.current().r#type != Type(')' as i32) {
                let suggestion = if self.lexer.current().r#type == Type('=' as i32) {
                    Some("; did you mean to use '{' when defining a table?")
                } else {
                    None
                };

                self.expect_match_and_consume_fail(Type(')' as i32), &match_paren, suggestion);
                end = self.lexer.previous_location().end;
            } else {
                close_paren_found = true;
                self.next_lexeme();
            }

            let expr_group = unsafe {
                (*self.allocator).alloc(AstExprGroup::new(Location::new(start, end), expr))
            };

            if luaur_common::FFlag::LuauCstExprGroup.get() && self.options.store_cst_data {
                let close_pos = if close_paren_found {
                    self.lexer.previous_location().begin
                } else {
                    Position::missing()
                };
                let cst_node = unsafe { (*self.allocator).alloc(CstExprGroup::new(close_pos)) };
                self.cst_node_map.try_insert(
                    expr_group as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }

            expr_group as *mut AstExpr
        } else {
            self.parse_name_expr("expression")
        }
    }
}
