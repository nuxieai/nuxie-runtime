use crate::records::allocator::Allocator;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_if_else::AstExprIfElse;
use crate::records::cst_expr_if_else::CstExprIfElse;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;

impl Parser {
    pub(crate) fn parse_if_else_expr(&mut self) -> *mut AstExpr {
        let mut has_else = false;
        let start = self.lexer.current().location;

        self.next_lexeme();

        let condition = self.parse_expr_i32(0);

        let has_then = self.expect_and_consume_type(Type::ReservedThen, "if then else expression");
        let then_position = if has_then {
            self.lexer.previous_location().begin
        } else {
            Position::missing()
        };

        let true_expr = self.parse_expr_i32(0);
        let mut false_expr: *mut AstExpr = core::ptr::null_mut();

        let else_position = self.lexer.current().location.begin;
        let mut is_else_if = false;

        if self.lexer.current().r#type == Type::ReservedElseif {
            let old_recursion_count = self.recursion_counter;
            self.increment_recursion_counter("expression");
            has_else = true;
            false_expr = self.parse_if_else_expr();
            self.recursion_counter = old_recursion_count;
            is_else_if = true;
        } else {
            has_else = self.expect_and_consume_type(Type::ReservedElse, "if then else expression");
            false_expr = self.parse_expr_i32(0);
        }

        let end = unsafe { (*false_expr).base.location };

        let node = unsafe {
            (*self.allocator).alloc(AstExprIfElse::new(
                Location::new(start.begin, end.end),
                condition,
                has_then,
                true_expr,
                has_else,
                false_expr,
            ))
        };

        if self.options.store_cst_data {
            let cst_node = unsafe {
                (*self.allocator).alloc(CstExprIfElse::new(
                    then_position,
                    else_position,
                    is_else_if,
                ))
            };
            self.cst_node_map.try_insert(
                node as *mut crate::records::ast_node::AstNode,
                cst_node as *mut crate::records::cst_node::CstNode,
            );
        }

        node as *mut AstExpr
    }
}
