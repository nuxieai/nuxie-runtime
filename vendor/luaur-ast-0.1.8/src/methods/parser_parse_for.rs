//! Node: `cxx:Method:Luau.Ast:Ast/src/Parser.cpp:726:parseFor`
//!
//! Faithful port of `Parser::parseFor` — both numeric `for i = a, b [, c] do`
//! and generic `for a, b in exprs do` loops. The first binding is parsed before
//! the `=`/`in` is known; numeric loops push the single control variable, while
//! generic loops parse a binding list and an expression list, copying both into
//! arena `AstArray`s for `AstStatForIn`. CST positions (commas / `=` / colon
//! annotations) are recorded only under `store_cst_data`.

use crate::records::allocator::Allocator;
use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_for::AstStatFor;
use crate::records::ast_stat_for_in::AstStatForIn;
use crate::records::cst_node::CstNode;
use crate::records::cst_stat_for::CstStatFor;
use crate::records::cst_stat_for_in::CstStatForIn;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_for(&mut self) -> *mut AstStat {
        let start = self.lexer.current().location;

        self.next_lexeme(); // for

        let varname = self.parse_binding(false);

        if self.lexer.current().r#type == Type(b'=' as i32) {
            let equals_position = self.lexer.current().location.begin;
            self.next_lexeme();

            let from = self.parse_expr_i32(0);

            let has_end_comma = self.expect_and_consume_char(',', "index range");
            let end_comma_position = if has_end_comma {
                self.lexer.previous_location().begin
            } else {
                Position::missing()
            };

            let to = self.parse_expr_i32(0);

            let mut step_comma_position = Position::missing();
            let mut step: *mut AstExpr = core::ptr::null_mut();

            if self.lexer.current().r#type == Type(b',' as i32) {
                step_comma_position = self.lexer.current().location.begin;
                self.next_lexeme();

                step = self.parse_expr_i32(0);
            }

            let match_do = *self.lexer.current();
            let has_do = self.expect_and_consume_type(Type::ReservedDo, "for loop");

            let locals_begin = self.save_locals();

            unsafe {
                (*self.function_stack.last_mut().unwrap()).loop_depth += 1;
            }

            let var = self.push_local(&varname);

            let body = self.parse_block();

            unsafe {
                (*self.function_stack.last_mut().unwrap()).loop_depth -= 1;
            }

            self.restore_locals(locals_begin);

            let end = self.lexer.current().location;

            let has_end =
                self.expect_match_end_and_consume(Type::ReservedEnd, &MatchLexeme::new(&match_do));
            unsafe {
                (*body).has_end = has_end;
            }

            let node = unsafe {
                (*self.allocator).alloc(AstStatFor::new(
                    Location::new(start.begin, end.end),
                    var,
                    from,
                    to,
                    step,
                    body,
                    has_do,
                    match_do.location,
                ))
            };

            if self.options.store_cst_data {
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstStatFor::new(
                        varname.colon_position,
                        equals_position,
                        end_comma_position,
                        step_comma_position,
                    ))
                };
                self.cst_node_map
                    .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
            }

            node as *mut AstStat
        } else {
            let mut names = TempVector::new(&mut self.scratch_binding);
            let mut vars_comma_position: AstArray<Position> = AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            };
            names.push_back(varname);

            if self.lexer.current().r#type == Type(b',' as i32) {
                if self.options.store_cst_data {
                    let mut initial_comma_position = self.lexer.current().location.begin;
                    self.next_lexeme();
                    let _ = self.parse_binding_list(
                        &mut names,
                        false,
                        &mut vars_comma_position,
                        &mut initial_comma_position,
                        core::ptr::null_mut(),
                        false,
                    );
                } else {
                    self.next_lexeme();
                    let _ = self.parse_binding_list(
                        &mut names,
                        false,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        false,
                    );
                }
            }

            let in_location = self.lexer.current().location;
            let has_in = self.expect_and_consume_type(Type::ReservedIn, "for loop");

            let mut values = TempVector::new(&mut self.scratch_expr);
            let mut values_comma_positions = TempVector::new(&mut self.scratch_position);
            self.parse_expr_list(
                &mut values,
                if self.options.store_cst_data {
                    Some(&mut values_comma_positions)
                } else {
                    None
                },
            );

            let match_do = *self.lexer.current();
            let has_do = self.expect_and_consume_type(Type::ReservedDo, "for loop");

            let locals_begin = self.save_locals();

            unsafe {
                (*self.function_stack.last_mut().unwrap()).loop_depth += 1;
            }

            let mut vars = TempVector::new(&mut self.scratch_local);

            for i in 0..names.size() {
                let local = self.push_local(names.operator_index(i));
                vars.push_back(local);
            }

            let body = self.parse_block();

            unsafe {
                (*self.function_stack.last_mut().unwrap()).loop_depth -= 1;
            }

            self.restore_locals(locals_begin);

            let end = self.lexer.current().location;

            let has_end =
                self.expect_match_end_and_consume(Type::ReservedEnd, &MatchLexeme::new(&match_do));
            unsafe {
                (*body).has_end = has_end;
            }

            let vars_array = self.copy_temp_vector_t(&vars);
            let values_array = self.copy_temp_vector_t(&values);

            let node = unsafe {
                (*self.allocator).alloc(AstStatForIn::new(
                    Location::new(start.begin, end.end),
                    vars_array,
                    values_array,
                    body,
                    has_in,
                    in_location,
                    has_do,
                    match_do.location,
                ))
            };

            if self.options.store_cst_data {
                let annotation = self.extract_annotation_colon_positions(&names);
                let values_comma = self.copy_temp_vector_t(&values_comma_positions);
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstStatForIn::new(
                        annotation,
                        vars_comma_position,
                        values_comma,
                    ))
                };
                self.cst_node_map
                    .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
            }

            node as *mut AstStat
        }
    }
}
