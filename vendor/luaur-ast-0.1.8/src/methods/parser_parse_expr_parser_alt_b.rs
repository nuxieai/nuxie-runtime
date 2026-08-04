//! Node: `cxx:Method:Luau.Ast:Ast/src/Parser.cpp:3530:parseExpr`
//!
//! The precedence-climbing expression parser (`Parser::parseExpr(unsigned limit)`).
//! Operands come from `parse_assertion_expr` (and recursively from here for unary
//! operators); binary operators expand while their left priority exceeds `limit`,
//! right-associative ops (`^`, `..`) recurse at one-lower priority. All operand
//! and operator helpers (`parse_unary_op`/`parse_binary_op`/`check_*_confusables`)
//! are already ported; only this driver was a stub.

use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_binary::AstExprBinary;
use crate::records::ast_expr_unary::AstExprUnary;
use crate::records::binary_op_priority::BinaryOpPriority;
use crate::records::cst_expr_op::CstExprOp;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_expr_i32(&mut self, limit: u32) -> *mut AstExpr {
        // Per-operator {left, right} binding priorities, indexed by
        // `AstExprBinaryOp as usize` (same order as the C++ table).
        const fn p(left: u8, right: u8) -> BinaryOpPriority {
            BinaryOpPriority { left, right }
        }
        let binary_priority: [BinaryOpPriority; 16] = [
            p(6, 6),  // +
            p(6, 6),  // -
            p(7, 7),  // *
            p(7, 7),  // /
            p(7, 7),  // //
            p(7, 7),  // %
            p(10, 9), // ^ (right associative)
            p(5, 4),  // .. (right associative)
            p(3, 3),  // ~=
            p(3, 3),  // ==
            p(3, 3),  // <
            p(3, 3),  // <=
            p(3, 3),  // >
            p(3, 3),  // >=
            p(2, 2),  // and
            p(1, 1),  // or
        ];
        const UNARY_PRIORITY: u32 = 8;

        let start = self.lexer.current().location;

        // C++ parseExpr enforces the recursion limit here (the port dropped it
        // entirely, so deeply-nested expressions never hit LuauRecursionLimit).
        let old_recursion_count = self.recursion_counter;
        // this handles recursive calls to parse_sub_expr/parse_expr
        self.increment_recursion_counter("expression");

        let curr = *self.lexer.current();
        let mut uop = self.parse_unary_op(&curr);
        if uop.is_none() {
            uop = self.check_unary_confusables();
        }

        let mut expr: *mut AstExpr;
        if let Some(uop) = uop {
            let op_position = self.lexer.current().location.begin;
            self.next_lexeme();
            let subexpr = self.parse_expr_i32(UNARY_PRIORITY);
            let end = unsafe { (*subexpr).base.location.end };
            let node = unsafe {
                (*self.allocator).alloc(AstExprUnary::new(
                    Location::new(start.begin, end),
                    uop,
                    subexpr,
                ))
            };
            if self.options.store_cst_data {
                let cst_node = unsafe { (*self.allocator).alloc(CstExprOp::new(op_position)) };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }
            expr = node as *mut AstExpr;
        } else {
            expr = self.parse_assertion_expr();
        }

        // Expand while operators have priority higher than `limit`.
        let curr = *self.lexer.current();
        let mut op = self.parse_binary_op(&curr);
        if op.is_none() {
            op = self.check_binary_confusables(&binary_priority, limit);
        }

        while let Some(o) = op {
            if binary_priority[o as usize].left as u32 <= limit {
                break;
            }
            let op_position = self.lexer.current().location.begin;
            self.next_lexeme();
            let next = self.parse_expr_i32(binary_priority[o as usize].right as u32);
            let end = unsafe { (*next).base.location.end };
            let node = unsafe {
                (*self.allocator).alloc(AstExprBinary::new(
                    Location::new(start.begin, end),
                    o,
                    expr,
                    next,
                ))
            };
            if self.options.store_cst_data {
                let cst_node = unsafe { (*self.allocator).alloc(CstExprOp::new(op_position)) };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }
            expr = node as *mut AstExpr;
            let curr = *self.lexer.current();
            op = self.parse_binary_op(&curr);
            if op.is_none() {
                op = self.check_binary_confusables(&binary_priority, limit);
            }

            // note: while the parser isn't recursive here, we're generating
            // recursive structures of unbounded depth
            self.increment_recursion_counter("expression");
        }

        self.recursion_counter = old_recursion_count;

        expr
    }
}
