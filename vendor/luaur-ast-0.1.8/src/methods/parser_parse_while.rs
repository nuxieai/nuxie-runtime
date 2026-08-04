//! Node: `cxx:Method:Luau.Ast:Ast/src/Parser.cpp:620:parseWhile`
//!
//! Faithful port of `Parser::parseWhile` — `while exp do block end`. Tracks
//! loop depth on the current function frame around the body parse and records
//! the `do` keyword location for CST-free reconstruction.

use crate::records::allocator::Allocator;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_while::AstStatWhile;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_while(&mut self) -> *mut AstStat {
        let start = self.lexer.current().location;

        self.next_lexeme(); // while

        let cond = self.parse_expr_i32(0);

        let match_do = *self.lexer.current();
        let has_do = self.expect_and_consume_type(Type::ReservedDo, "while loop");

        unsafe {
            (*self.function_stack.last_mut().unwrap()).loop_depth += 1;
        }

        let body = self.parse_block();

        unsafe {
            (*self.function_stack.last_mut().unwrap()).loop_depth -= 1;
        }

        let end = self.lexer.current().location;

        let has_end =
            self.expect_match_end_and_consume(Type::ReservedEnd, &MatchLexeme::new(&match_do));
        unsafe {
            (*body).has_end = has_end;
        }

        unsafe {
            (*self.allocator).alloc(AstStatWhile::new(
                Location::new(start.begin, end.end),
                cond,
                body,
                has_do,
                match_do.location,
            )) as *mut AstStat
        }
    }
}
