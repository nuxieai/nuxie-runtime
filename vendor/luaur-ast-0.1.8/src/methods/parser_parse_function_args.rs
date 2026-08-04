//! Node: `cxx:Method:Luau.Ast:Ast/src/Parser.cpp:4068:parseFunctionArgs`
//!
//! Faithful port of `Parser::parseFunctionArgs` — the function-call argument
//! parser (`args ::= '(' [explist] ')' | tableconstructor | String`). Each
//! variant produces an `AstExprCall` node; the parenthesised branch collects
//! comma positions under `store_cst_data` and records CST open/close parens.

use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_call::AstExprCall;
use crate::records::ast_node::AstNode;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::cst_expr_call::CstExprCall;
use crate::records::cst_node::CstNode;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_function_args(&mut self, func: *mut AstExpr, self_: bool) -> *mut AstExpr {
        if self.lexer.current().r#type == Type('(' as i32) {
            let arg_start = self.lexer.current().location.end;

            if unsafe { (*func).base.location.end.line } != self.lexer.current().location.begin.line
            {
                self.report_ambiguous_call_error();
            }

            let match_paren =
                crate::records::match_lexeme::MatchLexeme::new(&*self.lexer.current());
            self.next_lexeme();

            let mut args = TempVector::new(&mut self.scratch_expr);
            let mut comma_positions = TempVector::new(&mut self.scratch_position);

            if self.lexer.current().r#type != Type(')' as i32) {
                self.parse_expr_list(
                    &mut args,
                    if self.options.store_cst_data {
                        Some(&mut comma_positions)
                    } else {
                        None
                    },
                );
            }

            let end = self.lexer.current().location;
            let arg_end = end.end;

            let closing_paren_found = self.expect_match_and_consume(')', &match_paren, true);

            let args_array = self.copy_temp_vector_t(&args);
            let explicit_types: AstArray<AstTypeOrPack> = AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            };

            let node = unsafe {
                (*self.allocator).alloc(AstExprCall::new(
                    Location::new((*func).base.location.begin, end.end),
                    func,
                    args_array,
                    self_,
                    explicit_types,
                    Location::new(arg_start, arg_end),
                ))
            };

            if self.options.store_cst_data {
                let comma_positions_array = self.copy_temp_vector_t(&comma_positions);
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstExprCall::new(
                        match_paren.position,
                        if closing_paren_found {
                            self.lexer.previous_location().begin
                        } else {
                            Position::missing()
                        },
                        comma_positions_array,
                    ))
                };
                self.cst_node_map
                    .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
            }

            node as *mut AstExpr
        } else if self.lexer.current().r#type == Type('{' as i32) {
            let arg_start = self.lexer.current().location.end;
            let expr = self.parse_table_constructor();
            let arg_end = self.lexer.previous_location().end;

            let exprs = self.copy_t_usize(&expr as *const *mut AstExpr, 1);
            let explicit_types: AstArray<AstTypeOrPack> = AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            };

            let node = unsafe {
                (*self.allocator).alloc(AstExprCall::new(
                    Location::new((*func).base.location.begin, (*expr).base.location.end),
                    func,
                    exprs,
                    self_,
                    explicit_types,
                    Location::new(arg_start, arg_end),
                ))
            };

            if self.options.store_cst_data {
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstExprCall::new(
                        Position::missing(),
                        Position::missing(),
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                    ))
                };
                self.cst_node_map
                    .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
            }

            node as *mut AstExpr
        } else if self.lexer.current().r#type == Type::RawString
            || self.lexer.current().r#type == Type::QuotedString
        {
            let arg_location = self.lexer.current().location;
            let expr = self.parse_string();

            let exprs = self.copy_t_usize(&expr as *const *mut AstExpr, 1);
            let explicit_types: AstArray<AstTypeOrPack> = AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            };

            let node = unsafe {
                (*self.allocator).alloc(AstExprCall::new(
                    Location::new((*func).base.location.begin, (*expr).base.location.end),
                    func,
                    exprs,
                    self_,
                    explicit_types,
                    arg_location,
                ))
            };

            if self.options.store_cst_data {
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstExprCall::new(
                        Position::missing(),
                        Position::missing(),
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                    ))
                };
                self.cst_node_map
                    .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
            }

            node as *mut AstExpr
        } else {
            self.report_function_args_error(func, self_)
        }
    }
}
