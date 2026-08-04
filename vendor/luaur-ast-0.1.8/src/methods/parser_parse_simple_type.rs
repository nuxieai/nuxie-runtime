//! Node: `cxx:Method:Luau.Ast:Ast/src/Parser.cpp:3184:parseSimpleType`
//!
//! Faithful port of `Parser::parseSimpleType` — the type-atom dispatch for
//! all simple (non-suffixed) type annotations: `nil`, `true`, `false`, string
//! singletons, interpolated-string errors, `typeof`, qualified names (`a.b`),
//! a (possibly generic) NAME reference such as `A` / `B<number>`, `{ table }`,
//! `( function )`, and `function`. Generic arguments recurse back through
//! `parse_type_params` -> `parse_type` -> `parse_simple_type`, so nested
//! references parse too. CST positions for reference parameters, `typeof`
//! parens, and string-quote details are recorded only under `store_cst_data`.

use crate::records::ast_array::AstArray;
use crate::records::ast_name::AstName;
use crate::records::ast_node::AstNode;
use crate::records::ast_type::AstType;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::ast_type_reference::AstTypeReference;
use crate::records::ast_type_singleton_bool::AstTypeSingletonBool;
use crate::records::ast_type_singleton_string::AstTypeSingletonString;
use crate::records::ast_type_typeof::AstTypeTypeof;
use crate::records::cst_expr_constant_string::CstExprConstantString;
use crate::records::cst_node::CstNode;
use crate::records::cst_type_reference::CstTypeReference;
use crate::records::cst_type_singleton_string::CstTypeSingletonString;
use crate::records::cst_type_typeof::CstTypeTypeof;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::name::Name;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;
use crate::rtti::AstNodeClass;

impl Parser {
    pub fn parse_simple_type(
        &mut self,
        allow_pack: bool,
        in_declaration_context: bool,
    ) -> AstTypeOrPack {
        self.increment_recursion_counter("type annotation");

        let start = self.lexer.current().location;

        let mut attributes = AstArray {
            data: core::ptr::null_mut(),
            size: 0,
        };

        if self.lexer.current().r#type == Type::Attribute
            || self.lexer.current().r#type == Type::AttributeOpen
        {
            if !in_declaration_context {
                return AstTypeOrPack {
                    r#type: self.report_type_error(
                        start,
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                        format_args!("attributes are not allowed in declaration context"),
                    ) as *mut AstType,
                    type_pack: core::ptr::null_mut(),
                };
            } else {
                attributes = Parser::parse_attributes(self);
                return self.parse_function_type(allow_pack, &attributes);
            }
        } else if self.lexer.current().r#type == Type::ReservedNil {
            self.next_lexeme();
            let node = unsafe {
                (*self.allocator).alloc(AstTypeReference::new(
                    start,
                    None,
                    self.name_nil,
                    None,
                    start,
                    false,
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                )) as *mut AstType
            };
            return AstTypeOrPack {
                r#type: node,
                type_pack: core::ptr::null_mut(),
            };
        } else if self.lexer.current().r#type == Type::ReservedTrue {
            self.next_lexeme();
            return AstTypeOrPack {
                r#type: unsafe {
                    (*self.allocator).alloc(AstTypeSingletonBool::new(start, true)) as *mut AstType
                },
                type_pack: core::ptr::null_mut(),
            };
        } else if self.lexer.current().r#type == Type::ReservedFalse {
            self.next_lexeme();
            return AstTypeOrPack {
                r#type: unsafe {
                    (*self.allocator).alloc(AstTypeSingletonBool::new(start, false)) as *mut AstType
                },
                type_pack: core::ptr::null_mut(),
            };
        } else if self.lexer.current().r#type == Type::RawString
            || self.lexer.current().r#type == Type::QuotedString
        {
            let mut style = crate::enums::quote_style_cst::QuoteStyle::QuotedDouble;
            let mut block_depth: u32 = 0;
            if self.options.store_cst_data {
                let (s, bd) = self.extract_string_details();
                style = s;
                block_depth = bd;
            }

            let mut original_string = AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            };

            if let Some(value) = self.parse_char_array(if self.options.store_cst_data {
                Some(&mut original_string)
            } else {
                None
            }) {
                let node = unsafe {
                    (*self.allocator).alloc(AstTypeSingletonString::new(start, value))
                        as *mut AstType
                };
                if self.options.store_cst_data {
                    let cst_node = unsafe {
                        (*self.allocator).alloc(CstTypeSingletonString::new(
                            original_string,
                            style,
                            block_depth,
                        ))
                    };
                    self.cst_node_map
                        .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
                }
                return AstTypeOrPack {
                    r#type: node,
                    type_pack: core::ptr::null_mut(),
                };
            } else {
                return AstTypeOrPack {
                    r#type: self.report_type_error(
                        start,
                        AstArray {
                            data: core::ptr::null_mut(),
                            size: 0,
                        },
                        format_args!("String literal contains malformed escape sequence"),
                    ) as *mut AstType,
                    type_pack: core::ptr::null_mut(),
                };
            }
        } else if self.lexer.current().r#type == Type::InterpStringBegin
            || self.lexer.current().r#type == Type::InterpStringSimple
        {
            self.parse_interp_string();
            return AstTypeOrPack {
                r#type: self.report_type_error(
                    start,
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    format_args!("Interpolated string literals cannot be used as types"),
                ) as *mut AstType,
                type_pack: core::ptr::null_mut(),
            };
        } else if self.lexer.current().r#type == Type::BrokenString {
            self.next_lexeme();
            return AstTypeOrPack {
                r#type: self.report_type_error(
                    start,
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    format_args!("Malformed string; did you forget to finish it?"),
                ) as *mut AstType,
                type_pack: core::ptr::null_mut(),
            };
        } else if self.lexer.current().r#type == Type::Name {
            let mut prefix: Option<AstName> = None;
            let mut prefix_point_position = Position::missing();
            let mut prefix_location: Option<Location> = None;
            let mut name = self.parse_name("type name");

            if self.lexer.current().r#type == Type(b'.' as i32) {
                prefix_point_position = self.lexer.current().location.begin;
                self.next_lexeme();

                prefix = Some(name.name);
                prefix_location = Some(name.location);
                name = self.parse_index_name("field name", &prefix_point_position);
            } else if self.lexer.current().r#type == Type::Dot3 {
                self.report(
                    self.lexer.current().location,
                    format_args!("Unexpected '...' after type name; type pack is not allowed in this context"),
                );
                self.next_lexeme();
            } else if name.name.operator_eq_c_char(c"typeof".as_ptr()) {
                let typeof_begin = *self.lexer.current();
                let open_paren_found = self.expect_and_consume_char('(', "typeof type");

                let expr = self.parse_expr_i32(0);

                let mut end = self.lexer.current().location;
                let close_paren_found = self.expect_match_and_consume(
                    ')',
                    &crate::records::match_lexeme::MatchLexeme::new(&typeof_begin),
                    false,
                );

                let node = unsafe {
                    (*self.allocator).alloc(AstTypeTypeof::new(
                        Location::new(start.begin, end.end),
                        expr,
                    )) as *mut AstType
                };
                if self.options.store_cst_data {
                    let cst_node = unsafe {
                        (*self.allocator).alloc(CstTypeTypeof::new(
                            if open_paren_found {
                                typeof_begin.location.begin
                            } else {
                                Position::missing()
                            },
                            if close_paren_found {
                                end.begin
                            } else {
                                Position::missing()
                            },
                        ))
                    };
                    self.cst_node_map
                        .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
                }
                return AstTypeOrPack {
                    r#type: node,
                    type_pack: core::ptr::null_mut(),
                };
            }

            let mut has_parameters = false;
            let mut parameters = AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            };
            let mut parameters_opening_position = Position::missing();
            let mut parameters_comma_positions = TempVector::new(&mut self.scratch_position);
            let mut parameters_closing_position = Position::missing();

            if self.lexer.current().r#type == Type::Less {
                has_parameters = true;
                if self.options.store_cst_data {
                    parameters = self.parse_type_params(
                        Some(&mut parameters_opening_position),
                        Some(&mut parameters_comma_positions),
                        Some(&mut parameters_closing_position),
                    );
                } else {
                    parameters = self.parse_type_params(None, None, None);
                }
            }

            let end = *self.lexer.previous_location();

            let node = unsafe {
                (*self.allocator).alloc(AstTypeReference::new(
                    Location::new(start.begin, end.end),
                    prefix,
                    name.name,
                    prefix_location,
                    name.location,
                    has_parameters,
                    parameters,
                )) as *mut AstType
            };
            if self.options.store_cst_data {
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstTypeReference::new(
                        prefix_point_position,
                        parameters_opening_position,
                        self.copy_temp_vector_t(&parameters_comma_positions),
                        parameters_closing_position,
                    ))
                };
                self.cst_node_map
                    .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
            }
            return AstTypeOrPack {
                r#type: node,
                type_pack: core::ptr::null_mut(),
            };
        } else if self.lexer.current().r#type == Type(b'{' as i32) {
            return AstTypeOrPack {
                r#type: self.parse_table_type(in_declaration_context),
                type_pack: core::ptr::null_mut(),
            };
        } else if self.lexer.current().r#type == Type('(' as i32)
            || self.lexer.current().r#type == Type::Less
        {
            return self.parse_function_type(
                allow_pack,
                &AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
            );
        } else if self.lexer.current().r#type == Type::ReservedFunction {
            self.next_lexeme();

            return AstTypeOrPack {
                r#type: self.report_type_error(
                    start,
                    AstArray {
                        data: core::ptr::null_mut(),
                        size: 0,
                    },
                    format_args!(
                        "Using 'function' as a type annotation is not supported, consider replacing with a function type annotation e.g. '(...any) -> ...any'"
                    ),
                ) as *mut AstType,
                type_pack: core::ptr::null_mut(),
            };
        }

        // For a missing type annotation, capture 'space' between last token and the next one
        let ast_error_location = Location::new(self.lexer.previous_location().end, start.begin);
        // The parse error includes the next lexeme to make it easier to display where the error is.
        let parse_error_location = Location::new(self.lexer.previous_location().end, start.end);
        AstTypeOrPack {
            r#type: self.report_missing_type_error(
                parse_error_location,
                ast_error_location,
                format_args!("Expected type, got {}", self.lexer.current().to_string()),
            ) as *mut AstType,
            type_pack: core::ptr::null_mut(),
        }
    }
}
