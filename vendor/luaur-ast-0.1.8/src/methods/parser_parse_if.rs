//! Node: `cxx:Method:Luau.Ast:Ast/src/Parser.cpp:559:parseIf`
//!
//! Faithful port of `Parser::parseIf` — `if exp then block {elseif/else} end`.
//! Chained `elseif` recurses through `parse_if` (guarded by the recursion
//! counter); a trailing `else` is parsed as a block whose begin is snapped to
//! the `else` keyword. `hasEnd` is propagated onto whichever block terminates
//! the chain so the pretty-printer / CST consumers see the real end span.

use crate::records::allocator::Allocator;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::ast_stat_if::AstStatIf;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use crate::rtti::ast_node_as;

impl Parser {
    pub fn parse_if(&mut self) -> *mut AstStat {
        let start = self.lexer.current().location;

        self.next_lexeme(); // if / elseif

        let cond = self.parse_expr_i32(0);

        let match_then = *self.lexer.current();
        let mut then_location: Option<Location> = None;
        if self.expect_and_consume_type(Type::ReservedThen, "if statement") {
            then_location = Some(match_then.location);
        }

        let thenbody = self.parse_block();

        let mut elsebody: *mut AstStat = core::ptr::null_mut();
        let mut end = start;
        let mut else_location: Option<Location> = None;

        if self.lexer.current().r#type == Type::ReservedElseif {
            unsafe {
                (*thenbody).has_end = true;
            }
            let old_recursion_count = self.recursion_counter;
            self.increment_recursion_counter("elseif");
            else_location = Some(self.lexer.current().location);
            elsebody = self.parse_if();
            end = unsafe { (*elsebody).base.location };
            self.recursion_counter = old_recursion_count;
        } else {
            let mut match_then_else = match_then;

            if self.lexer.current().r#type == Type::ReservedElse {
                unsafe {
                    (*thenbody).has_end = true;
                }
                else_location = Some(self.lexer.current().location);
                match_then_else = *self.lexer.current();
                self.next_lexeme();

                let else_block = self.parse_block();
                unsafe {
                    (*else_block).base.base.location.begin = match_then_else.location.end;
                }
                elsebody = else_block as *mut AstStat;
            }

            end = self.lexer.current().location;

            let has_end = self.expect_match_end_and_consume(
                Type::ReservedEnd,
                &MatchLexeme::new(&match_then_else),
            );

            if !elsebody.is_null() {
                let else_block = unsafe { ast_node_as::<AstStatBlock>(elsebody as *mut AstNode) };
                if !else_block.is_null() {
                    unsafe {
                        (*else_block).has_end = has_end;
                    }
                }
            } else {
                unsafe {
                    (*thenbody).has_end = has_end;
                }
            }
        }

        unsafe {
            (*self.allocator).alloc(AstStatIf::new(
                Location::new(start.begin, end.end),
                cond,
                thenbody,
                elsebody,
                then_location,
                else_location,
            )) as *mut AstStat
        }
    }
}
