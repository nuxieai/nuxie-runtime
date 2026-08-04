use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_interp_string::AstExprInterpString;
use crate::records::cst_expr_interp_string::CstExprInterpString;
use crate::records::lexer::Lexer;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;
use core::ffi::c_char;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Parser {
    pub fn parse_interp_string(&mut self) -> *mut AstExpr {
        let mut strings = crate::records::temp_vector::TempVector::new(&mut self.scratch_string);
        let mut source_strings =
            crate::records::temp_vector::TempVector::new(&mut self.scratch_string_2);
        let mut string_positions =
            crate::records::temp_vector::TempVector::new(&mut self.scratch_position);
        let mut expressions = crate::records::temp_vector::TempVector::new(&mut self.scratch_expr);

        let start_location = self.lexer.current().location;
        let mut end_location = start_location;

        loop {
            let current_lexeme = *self.lexer.current();
            LUAU_ASSERT!(
                current_lexeme.r#type == crate::records::lexeme::Type::InterpStringBegin
                    || current_lexeme.r#type == crate::records::lexeme::Type::InterpStringMid
                    || current_lexeme.r#type == crate::records::lexeme::Type::InterpStringEnd
                    || current_lexeme.r#type == crate::records::lexeme::Type::InterpStringSimple
            );

            end_location = current_lexeme.location;

            let length = current_lexeme.get_length() as usize;
            let data_ptr = unsafe { current_lexeme.data.data } as *const c_char;
            let bytes = unsafe { core::slice::from_raw_parts(data_ptr as *const u8, length) };

            if self.options.store_cst_data {
                let source_string = self.copy_bytes(bytes);
                source_strings.push_back(source_string);
                string_positions.push_back(current_lexeme.location.begin);
            }

            let mut data = bytes.to_vec();
            if !Lexer::fixup_quoted_bytes(&mut data) {
                self.next_lexeme();
                return self.report_expr_error(
                    Location::new(start_location.begin, end_location.end),
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    format_args!("Interpolated string literal contains malformed escape sequence"),
                ) as *mut AstExpr;
            }

            let chars = self.copy_bytes(&data);
            self.next_lexeme();
            strings.push_back(chars);

            if current_lexeme.r#type == crate::records::lexeme::Type::InterpStringEnd
                || current_lexeme.r#type == crate::records::lexeme::Type::InterpStringSimple
            {
                break;
            }

            let mut error_while_checking = false;

            match self.lexer.current().r#type {
                crate::records::lexeme::Type::InterpStringMid
                | crate::records::lexeme::Type::InterpStringEnd => {
                    error_while_checking = true;
                    self.next_lexeme();
                    expressions.push_back(self.report_expr_error(
                        end_location,
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                        format_args!(
                            "Malformed interpolated string, expected expression inside '{{}}'"
                        ),
                    ) as *mut AstExpr);
                }
                crate::records::lexeme::Type::BrokenString => {
                    error_while_checking = true;
                    self.next_lexeme();
                    expressions.push_back(self.report_expr_error(
                        end_location,
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                        format_args!("Malformed interpolated string; did you forget to add a '`'?"),
                    ) as *mut AstExpr);
                }
                _ => {
                    expressions.push_back(self.parse_expr_i32(0));
                }
            }

            if error_while_checking {
                break;
            }

            match self.lexer.current().r#type {
                crate::records::lexeme::Type::InterpStringBegin
                | crate::records::lexeme::Type::InterpStringMid
                | crate::records::lexeme::Type::InterpStringEnd => {}
                crate::records::lexeme::Type::BrokenInterpDoubleBrace => {
                    self.next_lexeme();
                    return self.report_expr_error(
                        end_location,
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                        format_args!("Double braces are not permitted within interpolated strings; did you mean '\\{{'?"),
                    ) as *mut AstExpr;
                }
                crate::records::lexeme::Type::BrokenString | crate::records::lexeme::Type::Eof => {
                    if self.lexer.current().r#type == crate::records::lexeme::Type::BrokenString {
                        self.next_lexeme();
                    }
                    let strings_array = self.copy_temp_vector_t(&strings);
                    let expressions_array = self.copy_temp_vector_t(&expressions);
                    let node = unsafe {
                        (*self.allocator).alloc(AstExprInterpString::new(
                            Location::new(start_location.begin, self.lexer.previous_location().end),
                            strings_array,
                            expressions_array,
                        ))
                    };

                    if self.options.store_cst_data {
                        let source_strings_array = self.copy_temp_vector_t(&source_strings);
                        let string_positions_array = self.copy_temp_vector_t(&string_positions);
                        let cst_node = unsafe {
                            (*self.allocator).alloc(CstExprInterpString::new(
                                source_strings_array,
                                string_positions_array,
                            ))
                        };
                        self.cst_node_map.try_insert(
                            node as *mut crate::records::ast_node::AstNode,
                            cst_node as *mut crate::records::cst_node::CstNode,
                        );
                    }

                    if let Some(top) = self.lexer.peek_brace_stack_top() {
                        if top == crate::enums::brace_type::BraceType::InterpolatedString {
                            self.report_location_c_char_item(
                                *self.lexer.previous_location(),
                                format_args!(
                                    "Malformed interpolated string; did you forget to add a '}}'?"
                                ),
                            );
                        }
                    } else {
                        self.report_location_c_char_item(
                            *self.lexer.previous_location(),
                            format_args!(
                                "Malformed interpolated string; did you forget to add a '`'?"
                            ),
                        );
                    }

                    return node as *mut AstExpr;
                }
                _ => {
                    return self.report_expr_error(
                        end_location,
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                        format_args!(
                            "Malformed interpolated string, got {}",
                            self.lexer.current().to_string()
                        ),
                    ) as *mut AstExpr;
                }
            }
        }

        let strings_array = self.copy_temp_vector_t(&strings);
        let expressions_array = self.copy_temp_vector_t(&expressions);
        let node = unsafe {
            (*self.allocator).alloc(AstExprInterpString::new(
                Location::new(start_location.begin, end_location.end),
                strings_array,
                expressions_array,
            ))
        };

        if self.options.store_cst_data {
            let source_strings_array = self.copy_temp_vector_t(&source_strings);
            let string_positions_array = self.copy_temp_vector_t(&string_positions);
            let cst_node = unsafe {
                (*self.allocator).alloc(CstExprInterpString::new(
                    source_strings_array,
                    string_positions_array,
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
